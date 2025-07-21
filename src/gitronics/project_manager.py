from dataclasses import dataclass
from pathlib import Path

import yaml

from gitronics.file_discovery import get_file_paths


@dataclass
class _Configuration:
    overrides: str | None
    envelope_structure: str | None
    envelopes: dict[str, str]
    source: str | None
    tallies: list[str] | None
    materials: list[str] | None
    transforms: list[str] | None


class ProjectManager:
    def __init__(self, project_path: Path):
        self.file_paths = get_file_paths(project_path)

    def build_model(self, configuration_name: str, write_path: Path):
        pass

    def read_configuration(self, configuration_name: str) -> _Configuration:
        if configuration_name not in self.file_paths:
            raise ValueError(f"Configuration file {configuration_name} not found.")
        conf_path = self.file_paths[configuration_name]

        with open(conf_path, encoding="utf-8") as infile:
            conf_dict = yaml.safe_load(infile)

        configuration = _Configuration(
            overrides=conf_dict.get("overrides"),
            envelope_structure=conf_dict.get("envelope_structure"),
            envelopes=conf_dict.get("envelopes", {}),
            source=conf_dict.get("source"),
            tallies=conf_dict.get("tallies"),
            materials=conf_dict.get("materials"),
            transforms=conf_dict.get("transformations"),
        )

        configuration = self._override_configuration(configuration)

        return configuration

    def _override_configuration(self, new_conf: _Configuration) -> _Configuration:
        if not new_conf.overrides:
            return new_conf

        base = self.read_configuration(new_conf.overrides)

        if new_conf.envelope_structure:
            base.envelope_structure = new_conf.envelope_structure
        if new_conf.envelopes:
            base.envelopes.update(new_conf.envelopes)
        if new_conf.source:
            base.source = new_conf.source
        if isinstance(new_conf.tallies, list):
            base.tallies = new_conf.tallies
        if isinstance(new_conf.materials, list):
            base.materials = new_conf.materials
        if isinstance(new_conf.transforms, list):
            base.transforms = new_conf.transforms

        return base

    def check_project(self):
        self.create_summary(write_path=Path(""))
        self.check_all_files_are_valid()
        self.check_all_configurations_are_valid()

    def create_summary(self, write_path: Path):
        pass

    def check_all_files_are_valid(self):
        pass

    def check_all_configurations_are_valid(self):
        pass
