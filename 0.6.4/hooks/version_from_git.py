"""MkDocs hook to derive docs version from Git tags.

This keeps the Material version label aligned with the repository state
when running local builds/serve and in CI.
"""

from __future__ import annotations

import re
import subprocess
from typing import Optional

_TAG_RE = re.compile(r"^v?(\d+\.\d+\.\d+)$")


def _git_semver() -> Optional[str]:
    """Return the nearest semantic version tag, normalized without leading 'v'."""
    commands = [
        ["git", "describe", "--tags", "--abbrev=0", "--match", "v[0-9]*.[0-9]*.[0-9]*"],
        ["git", "describe", "--tags", "--abbrev=0", "--match", "[0-9]*.[0-9]*.[0-9]*"],
    ]

    for command in commands:
        try:
            result = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            continue

        tag = result.stdout.strip()
        match = _TAG_RE.match(tag)
        if match:
            return match.group(1)

    return None


def on_config(config, **kwargs):
    """Inject Git-derived default docs version for Material + mike."""
    version = _git_semver()
    if not version:
        return config

    extra = config.setdefault("extra", {})
    version_cfg = extra.get("version")
    if isinstance(version_cfg, dict):
        version_cfg["default"] = version

    return config
