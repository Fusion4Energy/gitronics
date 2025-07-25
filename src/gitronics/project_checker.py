import logging
import re
from dataclasses import dataclass

from gitronics.file_discovery import get_all_file_paths
from gitronics.helpers import ALLOWED_SUFFIXES, Config, GitronicsError
from gitronics.project_manager import ProjectManager

PLACEHOLDER_PAT = re.compile(r"\$\s+FILL\s*=\s*(\w+)\s*")


@dataclass
class ProjectChecker:
    project_manager: ProjectManager

    # @classmethod
    # def check_project(cls):
    #     cls.create_summary(write_path=Path(""))
    #     cls.check_all_files_are_valid()
    #     cls.check_all_configurations_are_valid()

    # @classmethod
    # def check_all_files_are_valid(cls):
    #     pass

    # @classmethod
    # def check_all_configurations_are_valid(cls):
    #     pass

    # @classmethod
    # def create_summary(cls, write_path: Path):
    #     pass

    def check_configuration(self, config: Config) -> None:
        logging.info("Checking project configuration.")
        self._check_files_in_the_project()
        self._check_envelope_structure(config)
        self._check_envelopes(config)
        self._check_fillers(config)
        self._check_source(config)
        self._check_tallies(config)
        self._check_materials(config)
        self._check_transforms(config)
        self._trigger_warnings(config)

    def _check_files_in_the_project(self) -> None:
        """Checks that the files have no duplicate names and that the metadata files
        exist for MCNP files."""
        project_root = self.project_manager.project_root

        all_paths = get_all_file_paths(project_root)
        names = []
        for path in all_paths:
            if path.suffix not in ALLOWED_SUFFIXES:
                continue
            name = path.stem
            if name in names:
                raise GitronicsError(f"Duplicate file name found: {name}")
            names.append(name)
            if path.suffix == ".mcnp" and not path.with_suffix(".metadata").exists():
                raise GitronicsError(f"Metadata file not found for: {path}")

    def _check_envelope_structure(self, config: Config) -> None:
        """Checks that the envelope structure is defined in the configuration and that
        it exists."""
        if not config.envelope_structure:
            raise GitronicsError(
                "Envelope structure is not defined in the configuration."
            )
        if config.envelope_structure not in self.project_manager.file_paths:
            raise GitronicsError(
                f"Envelope structure file {config.envelope_structure} not found "
                "in the project."
            )

    def _check_envelopes(self, config: Config) -> None:
        """Checks that, if there is an envelopes field, all the envelopes appear in the
        envelope structure. Prints a warning if there are envelopes that do not appear
        in the configuration."""
        if not config.envelopes:
            return

        envelope_structure_path = self.project_manager.file_paths[
            config.envelope_structure
        ]
        with open(envelope_structure_path, encoding="utf-8") as infile:
            text = infile.read()
        envelope_names_in_structure = set()
        for line in text.splitlines():
            placeholder_match = PLACEHOLDER_PAT.search(line)
            if placeholder_match:
                envelope_name = placeholder_match.group(1)
                envelope_names_in_structure.add(envelope_name)

        for envelope_name in config.envelopes.keys():
            if envelope_name not in envelope_names_in_structure:
                raise GitronicsError(
                    f"Envelope {envelope_name} not found in the envelope structure."
                )

        empty_envelopes = envelope_names_in_structure.difference(
            config.envelopes.keys()
        )
        if empty_envelopes:
            logging.warning(
                "There are %d empty envelopes in the structure.", len(empty_envelopes)
            )

    def _check_fillers(self, config: Config) -> None:
        """Check that all the fillers exist and that their metadata includes the
        necessary transformation."""
        for envelope_name, filler_name in config.envelopes.items():
            if not filler_name:
                continue
            if filler_name not in self.project_manager.file_paths:
                raise GitronicsError(
                    f"Filler file {filler_name} not found in the project."
                )
            metadata = self.project_manager.get_metadata(filler_name)
            try:
                metadata["transformations"][envelope_name]
            except (KeyError, TypeError):
                raise GitronicsError(
                    f"Transformation for envelope {envelope_name} not found in filler"
                    f" model {filler_name} metadata."
                )

    def _check_source(self, config: Config) -> None:
        if config.source:
            if config.source not in self.project_manager.file_paths:
                raise GitronicsError(
                    f"Source file {config.source} not found in the project."
                )

    def _check_tallies(self, config: Config) -> None:
        if config.tallies:
            for tally in config.tallies:
                if tally not in self.project_manager.file_paths:
                    raise GitronicsError(
                        f"Tally file {tally} not found in the project."
                    )

    def _check_materials(self, config: Config) -> None:
        if config.materials:
            for material in config.materials:
                if material not in self.project_manager.file_paths:
                    raise GitronicsError(
                        f"Material file {material} not found in the project."
                    )

    def _check_transforms(self, config: Config) -> None:
        if config.transforms:
            for transform in config.transforms:
                if transform not in self.project_manager.file_paths:
                    raise GitronicsError(
                        f"Transform file {transform} not found in the project."
                    )

    def _trigger_warnings(self, config: Config) -> None:
        if not config.source:
            logging.warning("No source included in the configuration!")
        if not config.materials or len(config.materials) == 0:
            logging.warning("No materials included in the configuration!")
