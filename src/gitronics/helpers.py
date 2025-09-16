from dataclasses import dataclass
from pathlib import Path

ALLOWED_SUFFIXES = {".mcnp", ".transform", ".mat", ".source", ".tally", ".yaml", ".yml"}

TYPE_BY_SUFFIX = {
    ".mcnp": "Geometry",
    ".transform": "Transform",
    ".mat": "Material",
    ".source": "Source",
    ".tally": "Tally",
    ".yaml": "Configuration",
    ".yml": "Configuration",
}


@dataclass
class ProjectParameters:
    root_folder_path: Path
    write_path: Path
    extra_metadata_fields: list[str] | None = None


@dataclass
class Config:
    name: str
    overrides: str | None
    envelope_structure: str
    envelopes: dict[str, str]
    source: str | None
    tallies: list[str] | None
    materials: list[str] | None
    transforms: list[str] | None


class GitronicsError(Exception):
    """Base class for all Gitronics exceptions."""
