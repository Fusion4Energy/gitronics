import logging
import re
from dataclasses import dataclass, field
from pathlib import Path

import polars as pl
from xlsxwriter import Workbook  # type: ignore

from gitronics.file_discovery import get_all_file_paths
from gitronics.helpers import ALLOWED_SUFFIXES, TYPE_BY_SUFFIX, Config, GitronicsError
from gitronics.project_manager import ProjectManager

PLACEHOLDER_PAT = re.compile(r"\$\s+FILL\s*=\s*(\w+)\s*")


@dataclass
class ConfigSummaryTables:
    configuration_and_structure: pl.DataFrame
    envelopes: pl.DataFrame
    data_files: pl.DataFrame


@dataclass
class SummaryData:
    all_files_info: pl.DataFrame | None = None
    config_summaries: dict[str, ConfigSummaryTables] = field(default_factory=dict)


class ProjectChecker:
    def __init__(self, project_manager: ProjectManager):
        self.project_manager = project_manager
        self.summary_data = SummaryData()

    def check_project(
        self, write_path: Path, extra_metadata_fields: list[str] | None = None
    ) -> None:
        """Checks the whole project for potential issues and creates a summary with
        all the files. It also checks the validity of all the configurations."""
        logging.info("Checking the validity of the whole project")
        file_paths = self._get_file_paths()
        self._check_no_duplicate_names(file_paths)
        self._check_metadata_files_exist_for_mcnp_models(file_paths)
        self._update_summary_data_with_all_files_info(file_paths, extra_metadata_fields)
        self._check_all_configurations(file_paths)
        self._write_excel_summary(write_path)

    def _check_all_configurations(self, paths: list[Path]) -> None:
        """Check all the configurations found in the project (files with .yaml or .yml
        suffix)."""
        configuration_files = [
            path for path in paths if path.suffix in {".yaml", ".yml"}
        ]
        for config_path in configuration_files:
            configuration_name = config_path.stem
            config = self.project_manager.read_configuration(configuration_name)
            self.check_configuration(config)

    def check_configuration(self, config: Config) -> None:
        logging.info("Checking configuration: %s", config.name)
        self._check_envelope_structure(config)
        self._check_envelopes(config)
        self._check_fillers(config)
        self._check_source(config)
        self._check_tallies(config)
        self._check_materials(config)
        self._check_transforms(config)
        self._trigger_warnings(config)
        self._update_summary_data_with_config(config)

    def _get_file_paths(self) -> list[Path]:
        paths = []
        all_file_paths = get_all_file_paths(self.project_manager.project_root)
        for path in all_file_paths:
            if path.suffix in ALLOWED_SUFFIXES:
                paths.append(path)
        return paths

    def _check_no_duplicate_names(self, paths: list[Path]) -> None:
        names = set()
        for path in paths:
            name = path.stem
            if name in names:
                raise GitronicsError(f"Duplicate file name found: {name}")
            names.add(name)

    def _check_metadata_files_exist_for_mcnp_models(self, paths: list[Path]) -> None:
        for path in paths:
            if path.suffix == ".mcnp" and not path.with_suffix(".metadata").exists():
                raise GitronicsError(f"Metadata file not found for: {path}")

    def _update_summary_data_with_all_files_info(
        self, paths: list[Path], extra_metadata_fields: list[str] | None = None
    ) -> None:
        data = []
        for path in paths:
            relative_path = str(
                path.relative_to(self.project_manager.project_root.absolute()).parent
            )
            entry = {
                "Type": TYPE_BY_SUFFIX[path.suffix],
                "Name": path.stem,
                "Path": relative_path,
            }
            data.append(entry)
            if extra_metadata_fields:
                try:
                    metadata = self.project_manager.get_metadata(path.stem)
                except GitronicsError:
                    metadata = {}
                for field in extra_metadata_fields:
                    entry[field] = metadata.get(field, "")

        dataframe = pl.DataFrame(data).sort(["Type", "Path", "Name"])
        self.summary_data.all_files_info = dataframe

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
                "There are empty envelopes in the structure not accounted for in the "
                "configuration: %d",
                empty_envelopes,
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
        """Check that the source file exists (if defined)."""
        if config.source:
            if config.source not in self.project_manager.file_paths:
                raise GitronicsError(
                    f"Source file {config.source} not found in the project."
                )

    def _check_tallies(self, config: Config) -> None:
        """Check that the tally files exist (if defined)."""
        if config.tallies:
            for tally in config.tallies:
                if tally not in self.project_manager.file_paths:
                    raise GitronicsError(
                        f"Tally file {tally} not found in the project."
                    )

    def _check_materials(self, config: Config) -> None:
        """Check that the material files exist (if defined)."""
        if config.materials:
            for material in config.materials:
                if material not in self.project_manager.file_paths:
                    raise GitronicsError(
                        f"Material file {material} not found in the project."
                    )

    def _check_transforms(self, config: Config) -> None:
        """Check that the transform files exist (if defined)."""
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

    def _update_summary_data_with_config(self, config: Config) -> None:
        """Adds to the summary_data attribute the information from the given
        configuration. This data will be used to create the Excel summary."""
        table_configuration_and_structure = [
            {"Type": "Configuration", "Name": config.name},
            {"Type": "Envelope Structure", "Name": config.envelope_structure},
        ]

        table_envelopes = []
        if config.envelopes:
            for envelope_name, filler_name in config.envelopes.items():
                table_envelopes.append(
                    {"Envelope": envelope_name, "Filler": filler_name}
                )

        table_data_files = []
        if config.source:
            table_data_files.append({"Type": "Source", "Name": config.source})
        if config.tallies:
            for tally in config.tallies:
                table_data_files.append({"Type": "Tally", "Name": tally})
        if config.materials:
            for material in config.materials:
                table_data_files.append({"Type": "Material", "Name": material})
        if config.transforms:
            for transform in config.transforms:
                table_data_files.append({"Type": "Transform", "Name": transform})
        self.summary_data.config_summaries[config.name] = ConfigSummaryTables(
            pl.DataFrame(table_configuration_and_structure),
            pl.DataFrame(table_envelopes),
            pl.DataFrame(table_data_files),
        )

    def _write_excel_summary(self, write_path: Path) -> None:
        with Workbook(write_path / "summary.xlsx") as workbook:
            if self.summary_data.all_files_info is not None:
                self.summary_data.all_files_info.write_excel(
                    workbook,
                    worksheet="All files",
                    position=(0, 0),
                    autofit=True,
                    header_format={"bold": True},
                )

            for conf_name, tables in self.summary_data.config_summaries.items():
                tables.configuration_and_structure.write_excel(
                    workbook,
                    worksheet=conf_name,
                    position=(0, 0),
                    autofit=True,
                    header_format={"bold": True},
                )
                tables.envelopes.write_excel(
                    workbook,
                    worksheet=conf_name,
                    position=(4, 0),
                    autofit=True,
                    header_format={"bold": True},
                )
                tables.data_files.write_excel(
                    workbook,
                    worksheet=conf_name,
                    position=(
                        4,
                        tables.envelopes.width + 1,
                    ),
                    autofit=True,
                    header_format={"bold": True},
                )
