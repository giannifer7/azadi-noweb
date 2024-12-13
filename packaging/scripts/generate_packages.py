#!/usr/bin/env python3
"""Package generation script for multiple distributions."""

import hashlib
import re
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Any, Optional, Tuple, List
from urllib.parse import urljoin

import click
import jinja2
import requests


def parse_author(author_str: str) -> Tuple[str, str]:
    """Parse author string into (name, email)."""
    match = re.match(r"(.*?)\s*<(.+?)>", author_str)
    if not match:
        raise ValueError(f"Invalid author format: {author_str}")
    return match.group(1).strip(), match.group(2).strip()


def fetch_and_compute_checksums(url: str) -> Tuple[str, str]:
    """Fetch a file and compute its SHA256 and SHA512 checksums."""
    try:
        response = requests.get(url, stream=True)
        response.raise_for_status()

        sha256 = hashlib.sha256()
        sha512 = hashlib.sha512()

        for chunk in response.iter_content(chunk_size=8192):
            sha256.update(chunk)
            sha512.update(chunk)

        return sha256.hexdigest(), sha512.hexdigest()
    except requests.RequestException as e:
        raise RuntimeError(f"Failed to download source: {e}")


def validate_distribution_config(config: Dict[str, Any]) -> None:
    """Validate the distribution configuration."""
    required_sections = ["build", "distributions"]
    for section in required_sections:
        if section not in config:
            raise ValueError(f"Missing required section: {section}")

    for dist, deps in config.get("distributions", {}).get("dependencies", {}).items():
        if not isinstance(deps, list):
            raise ValueError(f"Dependencies for {dist} must be a list")


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
    sha256sum: Optional[str] = None
    sha512sum: Optional[str] = None

    @property
    def maintainer(self) -> str:
        """Format maintainer information as 'name <email>'."""
        return f"{self.maintainer_name} <{self.maintainer_email}>"

    @classmethod
    def from_cargo_toml(cls, path: Path, dist_config: Dict[str, Any]) -> "PackageMetadata":
        """Create metadata from Cargo.toml and distribution config."""
        with open(path, "rb") as f:
            cargo_data = tomllib.load(f)
        package = cargo_data.get("package", {})

        if not package:
            raise ValueError("No [package] section found in Cargo.toml")

        required_fields = ["name", "version", "description", "license", "authors"]
        missing = [field for field in required_fields if not package.get(field)]
        if missing:
            raise ValueError(f"Missing required fields in Cargo.toml: {', '.join(missing)}")

        authors = package.get("authors", [])
        if not authors:
            raise ValueError("No authors found in Cargo.toml")

        maintainer_name, maintainer_email = parse_author(authors[0])

        # Get homepage from either homepage or repository field
        homepage = package.get("homepage", package.get("repository", ""))
        if not homepage:
            raise ValueError("Either homepage or repository must be specified in Cargo.toml")

        return cls(
            package_name=package["name"],
            version=package["version"],
            description=package["description"],
            license=package["license"],
            maintainer_name=maintainer_name,
            maintainer_email=maintainer_email,
            repo_url=package.get("repository", homepage),  # Fallback to homepage if no repository
            homepage=homepage,
        )

    def compute_checksums(self) -> None:
        """Compute SHA256 and SHA512 checksums for the source tarball."""
        if not self.repo_url:
            raise ValueError("Repository URL is required for checksum computation")

        tarball_url = f"{self.repo_url}/archive/v{self.version}.tar.gz"
        self.sha256sum, self.sha512sum = fetch_and_compute_checksums(tarball_url)

    def get_distribution_info(self, dist_name: str, dist_config: Dict[str, Any]) -> Dict[str, Any]:
        """Get distribution-specific information."""
        dist_info = dist_config.get("distributions", {})
        return {
            "architectures": dist_info.get("arch", ["x86_64"]),
            "dependencies": dist_info.get("dependencies", {}).get(dist_name, []),
            "libc_variant": dist_info.get("libc", {}).get(dist_name),
        }


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
            self.config = tomllib.load(f)
        validate_distribution_config(self.config)

    def clean_build_directory(self) -> None:
        """Clean all generated package files."""
        if self.build_dir.exists():
            shutil.rmtree(self.build_dir)
            print(f"Cleaned build directory: {self.build_dir}")

    def generate_all(self, metadata: PackageMetadata) -> None:
        """Generate all package formats."""
        formats = ["nix", "rpm", "deb", "arch", "alpine", "void-glibc", "void-musl"]
        for format_name in formats:
            self.generate_format(format_name, metadata)

    def get_output_path(self, format_name: str, metadata: PackageMetadata) -> Path:
        """Get the output path for a given format."""
        # Split format name for variants (e.g., 'void-musl' -> ('void', 'musl'))
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

        # Construct the build directory path
        build_path = self.build_dir / base_format
        if variant:
            build_path = build_path / variant

        return build_path / output_files[base_format]

    def generate_format(self, format_name: str, metadata: PackageMetadata) -> None:
        """Generate a specific package format."""
        # Use base format name for template
        base_format = format_name.split("-")[0]
        template = self.env.get_template(f"{base_format}.jinja2")

        output_path = self.get_output_path(format_name, metadata)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Get dict representation of metadata with proper maintainer format
        metadata_dict = {
            **metadata.__dict__,
            "maintainer": metadata.maintainer,
            **metadata.get_distribution_info(format_name, self.config),
        }

        content = template.render(metadata_dict)
        output_path.write_text(content)
        print(f"Generated {output_path}")


@click.command()
@click.option(
    "--cargo-toml",
    type=click.Path(exists=True),
    default="../../Cargo.toml",
    help="Path to Cargo.toml",
)
@click.option(
    "--compute-checksums/--skip-checksums",
    default=True,
    help="Whether to compute package checksums",
)
@click.option(
    "--formats",
    default="all",
    help="Comma-separated list of formats to generate (all,nix,rpm,deb,arch,alpine,void-glibc,void-musl)",
)
@click.option("--clean", is_flag=True, help="Clean build directory before generating packages")
def main(cargo_toml: str, compute_checksums: bool, formats: str, clean: bool) -> None:
    """Generate package files for various distributions."""
    project_root = Path(cargo_toml).parent

    try:
        generator = PackageGenerator(project_root)

        if clean:
            generator.clean_build_directory()

        # Load metadata
        metadata = PackageMetadata.from_cargo_toml(Path(cargo_toml), generator.config)

        # Compute checksums if requested
        if compute_checksums:
            metadata.compute_checksums()

        # Generate packages
        if formats == "all":
            generator.generate_all(metadata)
        else:
            for fmt in formats.split(","):
                generator.generate_format(fmt.strip(), metadata)

    except (ValueError, RuntimeError) as e:
        click.echo(f"Error: {e}", err=True)
        raise click.Abort()


if __name__ == "__main__":
    main()
