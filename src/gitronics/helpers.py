from dataclasses import dataclass


@dataclass
class Config:
    overrides: str | None
    envelope_structure: str
    envelopes: dict[str, str]
    source: str | None
    tallies: list[str] | None
    materials: list[str] | None
    transforms: list[str] | None
