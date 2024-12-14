# packaging/scripts/metadata.py
"""Package metadata handling."""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from .errors import ConfigError
from .types import PackageConfig, CargoToml

def parse_author(author_str: str) -> tuple[str, str]:
    """
    Parse author string in the format "Name <email>".
    Raises ConfigError if the format is invalid.
    """
    match = re.match(r"(.*?)\s*<(.+?)>", author_str)
    if not match:
        raise ConfigError(f"Invalid author format: {author_str}")
    return match.group(1).strip(), match.group(2).strip()

@dataclass
class PackageMetadata:
    """Metadata for a package, extracted from Cargo.toml and configuration."""
    package_name: str
    version: str
    description: str
    license: str
    maintainer_name: str
    maintainer_email: str
    repo_url: str
    homepage: str
    sha256sum: str | None = None
    sha512sum: str | None = None

    @property
    def maintainer(self) -> str:
        """Format maintainer information as 'name <email>'."""
        return f"{self.maintainer_name} <{self.maintainer_email}>"

    @classmethod
    def from_cargo_toml(cls, path: Path, dist_config: PackageConfig) -> PackageMetadata:
        """
        Create metadata from Cargo.toml and distribution config.
        Raises ConfigError if required fields are missing.
        """
        try:
            with open(path, "rb") as f:
                cargo_data: CargoToml = tomllib.load(f)
        except tomllib.TOMLDecodeError as e:
            raise ConfigError(f"Invalid Cargo.toml: {e}") from e
        except OSError as e:
            raise ConfigError(f"Could not read Cargo.toml: {e}") from e

        package = cargo_data.get("package", {})
        if not package:
            raise ConfigError("No [package] section found in Cargo.toml")

        authors = package.get("authors", [])
        if not authors:
            raise ConfigError("No authors found in Cargo.toml")

        required_fields = ["name", "version", "description", "license"]
        missing = [field for field in required_fields if field not in package]
        if missing:
            raise ConfigError(f"Missing required fields in Cargo.toml: {', '.join(missing)}")

        maintainer_name, maintainer_email = parse_author(authors[0])

        return cls(
            package_name=package["name"],
            version=package["version"],
            description=package["description"],
            license=package["license"],
            maintainer_name=maintainer_name,
            maintainer_email=maintainer_email,
            repo_url=package.get("repository", ""),
            homepage=package.get("homepage", package.get("repository", "")),
        )
