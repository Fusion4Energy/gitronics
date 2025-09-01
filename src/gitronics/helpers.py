from dataclasses import dataclass

ALLOWED_SUFFIXES = {".mcnp", ".transform", ".mat", ".source", ".tally", ".yaml", ".yml"}


@dataclass
class Config:
    overrides: str | None
    envelope_structure: str
    envelopes: dict[str, str]
    source: str | None
    tallies: list[str] | None
    materials: list[str] | None
    transforms: list[str] | None


class GitronicsError(Exception):
    """Base class for all Gitronics exceptions."""
