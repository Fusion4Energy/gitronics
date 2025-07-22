import re
from dataclasses import dataclass
from pathlib import Path

from gitronics.helpers import Config


@dataclass
class ProjectChecker:
    file_paths: dict[str, Path]

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
        self._check_envelope_structure(config)
        self._check_envelopes(config)
        self._check_source(config)
        self._check_tallies(config)
        self._check_materials(config)
        self._check_transforms(config)

    def _check_envelope_structure(self, config: Config) -> None:
        if not config.envelope_structure:
            raise ValueError("Envelope structure is not defined in the configuration.")
        if config.envelope_structure not in self.file_paths:
            raise ValueError(
                f"Envelope structure file {config.envelope_structure} not found "
                "in the project."
            )

    def _check_envelopes(self, config: Config) -> None:
        if not config.envelopes:
            return

        for filler_name in config.envelopes.values():
            if not filler_name:
                continue
            if filler_name not in self.file_paths:
                raise ValueError(f"Filler file {filler_name} not found in the project.")

        for envelope_name in config.envelopes:
            envelope_found = False
            placeholder_pat = re.compile(rf"^[^cC]*\$\s*FILL\s*=\s*{envelope_name}")
            assert config.envelope_structure
            with open(
                self.file_paths[config.envelope_structure], encoding="utf-8"
            ) as infile:
                for line in infile:
                    if placeholder_pat.match(line):
                        envelope_found = True
                        break
            if not envelope_found:
                raise ValueError(
                    f"Envelope {envelope_name} not found in the envelope structure."
                )

    def _check_source(self, config: Config) -> None:
        if config.source:
            if config.source not in self.file_paths:
                raise ValueError(
                    f"Source file {config.source} not found in the project."
                )

    def _check_tallies(self, config: Config) -> None:
        if config.tallies:
            for tally in config.tallies:
                if tally not in self.file_paths:
                    raise ValueError(f"Tally file {tally} not found in the project.")

    def _check_materials(self, config: Config) -> None:
        if config.materials:
            for material in config.materials:
                if material not in self.file_paths:
                    raise ValueError(
                        f"Material file {material} not found in the project."
                    )

    def _check_transforms(self, config: Config) -> None:
        if config.transforms:
            for transform in config.transforms:
                if transform not in self.file_paths:
                    raise ValueError(
                        f"Transform file {transform} not found in the project."
                    )
