"""
These functions take care of reading the configuration file and returning the paths of
the files that should be included in the model.
"""

import csv
import re
from dataclasses import dataclass
from pathlib import Path

import yaml


@dataclass
class EnvelopeData:
    filler: str | None
    transform: str | None


@dataclass
class Configuration:
    overrides: str | None
    envelope_structure: str | None
    envelopes: dict[str, EnvelopeData]
    source: str | None
    tallies: list[str]
    materials: list[str]
    transforms: list[str]


class ModelManager:
    def __init__(
        self,
        root_folder_path: Path,
        configuration_file_path: Path,
        project_summary_path: Path,
    ):
        self.root_folder_path = root_folder_path
        self.project_summary = self._read_project_summary(project_summary_path)
        self.configuration = self._read_configuration_file(configuration_file_path)

    def get_included_paths(self) -> set[Path]:
        paths = set()
        self._include_envelope_structure(paths)
        self._include_envelopes(paths)
        self._include_source(paths)
        self._include_tallies(paths)
        self._include_materials(paths)
        self._include_transforms(paths)
        return paths

    def get_universe_id(self, envelope_name: str) -> int:
        filler_model_path = self.project_summary[
            self.configuration.envelopes[envelope_name].filler
        ]
        with open(filler_model_path, encoding="utf-8") as infile:
            for line in infile:
                universe_match = re.search(r"[uU]\s*=\s*(\d+)", line)
                if universe_match:
                    return int(universe_match.group(1))
        raise ValueError(f"Universe ID not found in filler model {filler_model_path}")

    def _include_envelope_structure(self, paths: set[Path]) -> None:
        try:
            path = self.project_summary[self.configuration.envelope_structure]
            paths.add(path)
        except KeyError as e:
            raise KeyError(
                f"Envelope structure {self.configuration.envelope_structure} not found"
                " in project summary."
            ) from e

    def _include_envelopes(self, paths: set[Path]) -> None:
        for envelope in self.configuration.envelopes.values():
            if not envelope or not envelope.filler:
                continue
            try:
                path = self.project_summary[envelope.filler]
                paths.add(path)
            except KeyError as e:
                raise KeyError(
                    f"Filler {envelope.filler} not found in project summary."
                ) from e

    def _include_source(self, paths: set[Path]) -> None:
        try:
            path = self.project_summary[self.configuration.source]
            paths.add(path)
        except KeyError as e:
            raise KeyError(
                f"Source file {self.configuration.source} not found in project summary."
            ) from e

    def _include_tallies(self, paths: set[Path]) -> None:
        for tally in self.configuration.tallies:
            try:
                path = self.project_summary[tally]
                paths.add(path)
            except KeyError as e:
                raise KeyError(
                    f"Tally file {tally} not found in project summary."
                ) from e

    def _include_materials(self, paths: set[Path]) -> None:
        for material in self.configuration.materials:
            try:
                path = self.project_summary[material]
                paths.add(path)
            except KeyError as e:
                raise KeyError(
                    f"Material file {material} not found in project summary."
                ) from e

    def _include_transforms(self, paths: set[Path]) -> None:
        for transform in self.configuration.transforms:
            try:
                path = self.project_summary[transform]
                paths.add(path)
            except KeyError as e:
                raise KeyError(
                    f"Transform file {transform} not found in project summary."
                ) from e

    def _read_configuration_file(self, configuration_file_path: Path) -> Configuration:
        """
        Reads the configuration file.
        """
        with open(configuration_file_path, encoding="utf-8") as infile:
            conf_dict = yaml.safe_load(infile)

        envelopes_dict = conf_dict.get("envelopes", {})
        for envelope_name, envelope_data in envelopes_dict.items():
            if not envelope_data:
                continue
            envelopes_dict[envelope_name] = EnvelopeData(
                envelope_data.get("filler"), envelope_data.get("transform")
            )

        configuration = Configuration(
            overrides=conf_dict.get("overrides"),
            envelope_structure=conf_dict.get("envelope_structure"),
            envelopes=conf_dict.get("envelopes"),
            source=conf_dict.get("source"),
            tallies=conf_dict.get("tallies"),
            materials=conf_dict.get("materials"),
            transforms=conf_dict.get("transforms"),
        )
        configuration = self._override_base_configuration(configuration)
        return configuration

    def _override_base_configuration(
        self, configuration: Configuration
    ) -> Configuration:
        if not configuration.overrides:
            return configuration
        try:
            base_configuration = self._read_configuration_file(
                self.root_folder_path / self.project_summary[configuration.overrides]
            )
        except KeyError as e:
            raise KeyError(
                f"Configuration file {configuration.overrides} not found in project"
                " summary."
            ) from e

        if configuration.envelope_structure:
            base_configuration.envelope_structure = configuration.envelope_structure
        if configuration.envelopes:
            base_configuration.envelopes.update(configuration.envelopes)
        if configuration.source:
            base_configuration.source = configuration.source
        if isinstance(configuration.tallies, list):
            base_configuration.tallies = configuration.tallies
        if isinstance(configuration.materials, list):
            base_configuration.materials = configuration.materials
        if isinstance(configuration.transforms, list):
            base_configuration.transforms = configuration.transforms
        return base_configuration

    def _read_project_summary(self, project_summary_path: Path) -> dict:
        """
        Reads the project summary file and returns its content as a dictionary.
        """
        project_summary: dict[str, Path] = {}
        with open(project_summary_path, encoding="utf-8") as csvfile:
            reader = csv.DictReader(csvfile)
            for row in reader:
                name = row["Name"]
                if name in project_summary:
                    raise ValueError("The Name column has repeated values for %s", name)
                path = self.root_folder_path / Path(row["Relative path"])
                if not path.exists():
                    raise ValueError("The path %s does not exist", path)
                project_summary[name] = path
        return project_summary
