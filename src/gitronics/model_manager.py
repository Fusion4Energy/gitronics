"""
These functions take care of reading the configuration file and returning the paths of
the files that should be included in the model.
"""

import csv
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


@dataclass
class _EnvelopeData:
    filler: str | None
    transform: str | None


@dataclass
class _Configuration:
    overrides: str | None
    envelope_structure: str | None
    envelopes: dict[str, _EnvelopeData]
    source: str | None
    tallies: list[str]
    materials: list[str]
    transforms: list[str]


class ModelManager:
    """Manages the configuration and project summary files and their relevant
    information.
    """

    def __init__(
        self,
        root_folder_path: Path,
        configuration_file_path: Path,
        project_summary_path: Path,
    ):
        self.root_folder_path = root_folder_path
        self.project_summary = self._read_project_summary(project_summary_path)
        self.configuration = self._read_configuration_file(configuration_file_path)

    def get_included_paths(self) -> list[Path]:
        """Returns all the path files that will be included in the model according to
        the configuration and project summary files.
        """
        paths: list[Path] = []
        self._include_envelope_structure(paths)
        self._include_envelopes(paths)
        self._include_source(paths)
        self._include_tallies(paths)
        self._include_materials(paths)
        self._include_transforms(paths)
        return paths

    def get_universe_id(self, envelope_name: str) -> int:
        """Returns the universe ID of the envelope filler model."""
        filler_model_path = self.project_summary[
            self.configuration.envelopes[envelope_name].filler
        ]
        with open(filler_model_path, encoding="utf-8") as infile:
            for line in infile:
                universe_match = re.search(r"[uU]\s*=\s*(\d+)", line)
                if universe_match:
                    return int(universe_match.group(1))
        raise ValueError(f"Universe ID not found in filler model {filler_model_path}")

    def _include_envelope_structure(self, paths: list[Path]) -> None:
        try:
            path = self.project_summary[self.configuration.envelope_structure]
            paths.append(path)
        except KeyError as e:
            raise KeyError(
                f"Envelope structure {self.configuration.envelope_structure} not found"
                " in project summary."
            ) from e

    def _include_envelopes(self, paths: list[Path]) -> None:
        for envelope in self.configuration.envelopes.values():
            if not envelope or not envelope.filler:
                continue
            try:
                path = self.project_summary[envelope.filler]
                if path in paths:
                    continue
                paths.append(path)
            except KeyError as e:
                raise KeyError(
                    f"Filler {envelope.filler} not found in project summary."
                ) from e

    def _include_source(self, paths: list[Path]) -> None:
        try:
            path = self.project_summary[self.configuration.source]
            paths.append(path)
        except KeyError as e:
            raise KeyError(
                f"Source file {self.configuration.source} not found in project summary."
            ) from e

    def _include_tallies(self, paths: list[Path]) -> None:
        if not self.configuration.tallies:
            return
        for tally in self.configuration.tallies:
            try:
                path = self.project_summary[tally]
                if path in paths:
                    continue
                paths.append(path)
            except KeyError as e:
                raise KeyError(
                    f"Tally file {tally} not found in project summary."
                ) from e

    def _include_materials(self, paths: list[Path]) -> None:
        if not self.configuration.materials:
            return
        for material in self.configuration.materials:
            try:
                path = self.project_summary[material]
                if path in paths:
                    continue
                paths.append(path)
            except KeyError as e:
                raise KeyError(
                    f"Material file {material} not found in project summary."
                ) from e

    def _include_transforms(self, paths: list[Path]) -> None:
        if not self.configuration.transforms:
            return
        for transform in self.configuration.transforms:
            try:
                path = self.project_summary[transform]
                if path in paths:
                    continue
                paths.append(path)
            except KeyError as e:
                raise KeyError(
                    f"Transform file {transform} not found in project summary."
                ) from e

    def _read_configuration_file(self, configuration_file_path: Path) -> _Configuration:
        """
        Reads the configuration file.
        """
        with open(configuration_file_path, encoding="utf-8") as infile:
            conf_dict = yaml.safe_load(infile)

        envelopes_dict = conf_dict.get("envelopes", {})
        for envelope_name, envelope_data in envelopes_dict.items():
            if not envelope_data:
                continue
            envelopes_dict[envelope_name] = _EnvelopeData(
                envelope_data.get("filler"), envelope_data.get("transform")
            )

        configuration = _Configuration(
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
        self, configuration: _Configuration
    ) -> _Configuration:
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

    def _read_project_summary(self, project_summary_path: Path) -> dict[Any, Any]:
        """
        Reads the project summary file and returns its content as a dictionary.
        """
        project_summary: dict[str, Path] = {}
        with open(project_summary_path, encoding="utf-8") as csvfile:
            reader = csv.DictReader(csvfile)
            for row in reader:
                name = row["Name"]
                if name in project_summary:
                    raise ValueError(f"The Name column has repeated values for {name}")
                path = self.root_folder_path / Path(row["Relative path"])
                if not path.exists():
                    raise ValueError(f"The path {path} does not exist")
                project_summary[name] = path
        return project_summary
