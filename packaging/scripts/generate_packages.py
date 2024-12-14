# packaging/scripts/generate_packages.py
"""Package generation script for multiple distributions."""

from __future__ import annotations

import hashlib
import re
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path

import argparse
import jinja2
import requests

from typing import TypedDict, NotRequired

class DistributionDependencies(TypedDict):
    alpine: list[str]
    arch: list[str]
    void_glibc: list[str]
    void_musl: list[str]
    deb: list[str]
    rpm: list[str]

class LibcVariants(TypedDict):
    void_glibc: str
    void_musl: str

class DistributionsConfig(TypedDict):
    arch: list[str]
    dependencies: DistributionDependencies
    libc: LibcVariants

class BuildConfig(TypedDict):
    output_dir: str

class PackageConfig(TypedDict):
    build: BuildConfig
    distributions: DistributionsConfig

class CargoPackage(TypedDict):
    name: str
    version: str
    description: str
    license: str
    authors: list[str]
    repository: NotRequired[str]
    homepage: NotRequired[str]

class CargoToml(TypedDict):
    package: CargoPackage

def parse_author(author_str: str) -> tuple[str, str]:
    """Parse author string into (name, email)."""
    match = re.match(r"(.*?)\s*<(.+?)>", author_str)
    if not match:
        raise ValueError(f"Invalid author format: {author_str}")
    return match.group(1).strip(), match.group(2).strip()

def compute_checksums(url: str) -> tuple[str, str]:
    """Compute SHA256 and SHA512 checksums for a file."""
    response = requests.get(url, stream=True)
    response.raise_for_status()

    sha256 = hashlib.sha256()
    sha512 = hashlib.sha512()

    for chunk in response.iter_content(chunk_size=8192):
        sha256.update(chunk)
        sha512.update(chunk)

    return sha256.hexdigest(), sha512.hexdigest()

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
        """Create metadata from Cargo.toml and distribution config."""
        with open(path, "rb") as f:
            cargo_data: CargoToml = tomllib.load(f)
        package = cargo_data.get("package", {})

        if not package:
            raise ValueError("No [package] section found in Cargo.toml")

        authors = package.get("authors", [])
        if not authors:
            raise ValueError("No authors found in Cargo.toml")

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

class PackageGenerator:
    """Generate package files for multiple distributions."""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.packaging_dir = project_root / "packaging"
        self.templates_dir = self.packaging_dir / "templates"
        self.build_dir = self.packaging_dir / "build"

        self.env = jinja2.Environment(
            loader=jinja2.FileSystemLoader(self.templates_dir),
            undefined=jinja2.StrictUndefined,
            trim_blocks=True,
            lstrip_blocks=True,
        )

        config_file = self.packaging_dir / "config" / "metadata.toml"
        with open(config_file, "rb") as f:
            self.config: PackageConfig = tomllib.load(f)

        def get_output_path(self, format_name: str, metadata: PackageMetadata) -> Path:
            """Get the output path for a given format."""
            parts = format_name.split("-")
            base_format = parts[0]
            variant = parts[1] if len(parts) > 1 else None
    
            output_files = {
                "nix": "flake.nix",
                "rpm": f"{metadata.package_name}.spec",
                "deb": "control",
                "arch": "PKGBUILD",
                "alpine": "APKBUILD",
                "void": "template",
            }
    
            build_path = self.build_dir / base_format
            if variant:
                build_path = build_path / variant
    
            return build_path / output_files[base_format]
    
        def get_supported_distributions(self) -> list[str]:
            """Get list of supported distributions."""
            return list(self.config["distributions"]["dependencies"].keys())

        def generate_format(self, format_name: str, metadata: PackageMetadata) -> None:
            """Generate a specific package format."""
            # First generate any format-specific cargo configurations
            if format_name == "deb":
                self.generate_cargo_deb_config(metadata)
            
            # Use base format name for template
            base_format = format_name.split("-")[0]
            template = self.env.get_template(f"{base_format}.jinja2")
            
            output_path = self.get_output_path(format_name, metadata)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            metadata_dict = {
                **metadata.__dict__,
                "maintainer": metadata.maintainer,
                "checksum": metadata.sha256sum,
                **self.get_distribution_info(format_name, self.config),
            }
            
            content = template.render(metadata_dict)
            output_path.write_text(content)
            print(f"Generated {output_path}")
    
        def generate_cargo_deb_config(self, metadata: PackageMetadata) -> None:
            """Generate Cargo.deb.toml configuration."""
            if "deb" not in self.config["distributions"]["dependencies"]:
                return
    
            template = self.env.get_template("cargo.deb.toml.jinja2")
            output_path = self.build_dir / "deb" / "Cargo.deb.toml"
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            content = template.render(metadata.__dict__)
            output_path.write_text(content)
            print(f"Generated {output_path}")

