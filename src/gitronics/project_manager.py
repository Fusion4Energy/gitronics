import re
from pathlib import Path
from typing import Any

import yaml

from gitronics.file_discovery import get_file_paths
from gitronics.helpers import Config
from gitronics.project_checker import ProjectChecker


class ProjectManager:
    def __init__(self, project_path: Path):
        self.file_paths = get_file_paths(project_path)
        self.project_checker = ProjectChecker(self.file_paths)

    def get_included_paths(self, config: Config) -> list[Path]:
        self.project_checker.check_configuration(config)
        paths = []
        self._include_envelope_structure(paths, config)
        self._include_fillers(paths, config)
        self._include_source(paths, config)
        self._include_tallies(paths, config)
        self._include_materials(paths, config)
        self._include_transforms(paths, config)
        return paths

    def get_metadata(self, name: str) -> dict[str, Any]:
        if name not in self.file_paths:
            raise ValueError(f"File {name} not found in the project.")

        file_path = self.file_paths[name].with_suffix(".metadata")
        with open(file_path, encoding="utf-8") as infile:
            metadata = yaml.safe_load(infile) or {}

        return metadata

    def get_transformation(self, filler_name: str, envelope_name: str) -> str | None:
        metadata = self.get_metadata(filler_name)
        try:
            return metadata["transformations"][envelope_name]
        except KeyError:
            raise ValueError(
                f"Transformation for envelope {envelope_name} not found in "
                f"filler model {filler_name} metadata."
            )

    def get_universe_id(self, filler_name: str) -> int:
        """Returns the universe ID of the filler model."""
        filler_path = self.file_paths[filler_name]
        with open(filler_path, encoding="utf-8") as infile:
            for line in infile:
                universe_match = re.match(r"^[^cC\$]*\s*[uU]\s*=\s*(\d+)", line)
                if universe_match:
                    return int(universe_match.group(1))
        raise ValueError(f"Universe ID not found in filler model {filler_path}")

    def read_configuration(self, configuration_name: str) -> Config:
        if configuration_name not in self.file_paths:
            raise ValueError(f"Configuration file {configuration_name} not found.")
        conf_path = self.file_paths[configuration_name]

        with open(conf_path, encoding="utf-8") as infile:
            conf_dict = yaml.safe_load(infile)

        configuration = Config(
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

    def _override_configuration(self, new_conf: Config) -> Config:
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

    def _include_envelope_structure(self, paths: list[Path], config: Config) -> None:
        if config.envelope_structure in self.file_paths:
            paths.append(self.file_paths[config.envelope_structure])

    def _include_fillers(self, paths: list[Path], config: Config) -> None:
        if not config.envelopes:
            return
        for filler in config.envelopes.values():
            paths.append(self.file_paths[filler])

    def _include_source(self, paths: list[Path], config: Config) -> None:
        if not config.source:
            return
        paths.append(self.file_paths[config.source])

    def _include_tallies(self, paths: list[Path], config: Config) -> None:
        if not config.tallies:
            return
        for tally in config.tallies:
            paths.append(self.file_paths[tally])

    def _include_materials(self, paths: list[Path], config: Config) -> None:
        if not config.materials:
            return
        for material in config.materials:
            paths.append(self.file_paths[material])

    def _include_transforms(self, paths: list[Path], config: Config) -> None:
        if not config.transforms:
            return
        for transform in config.transforms:
            paths.append(self.file_paths[transform])
