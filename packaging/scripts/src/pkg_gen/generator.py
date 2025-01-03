# packaging/scripts/generator.py
"""Package generation and build coordination."""

from __future__ import annotations

import getpass
import subprocess
from pathlib import Path
import tomllib
from typing import NoReturn

import jinja2

from pkg_gen.errors import BuildError, ConfigError, TemplateError, ValidationError
from pkg_gen.metadata import PackageMetadata
from pkg_gen.pkgtypes import PackageConfig
from pkg_gen.builders import PackageBuilder, get_builder
from pkg_gen.utils.command import run_cmd, CommandError


class PackageGenerator:
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

        self.config = self._load_config()
        self._setup_build_dir()

    def _setup_build_dir(self) -> None:
        """Set up build directory with proper permissions."""
        self.build_dir.mkdir(parents=True, exist_ok=True)
        current_user = getpass.getuser()

        run_cmd(
            [
                "sudo",
                "chown",
                "-R",
                f"{current_user}:{current_user}",
                str(self.build_dir),
            ]
        )
        run_cmd(
            [
                "sudo",
                "chmod",
                "-R",
                "u+rw,g+rw",  # read/write for user and group
                str(self.build_dir),
            ]
        )

    def _load_config(self) -> PackageConfig:
        """Load and validate configuration."""
        config_file = self.packaging_dir / "config" / "metadata.toml"
        try:
            with open(config_file, "rb") as f:
                config: PackageConfig = tomllib.load(f)
        except (tomllib.TOMLDecodeError, OSError) as e:
            raise ConfigError(f"Failed to load config: {e}") from e

        self._validate_config(config)
        return config

    def _validate_config(self, config: PackageConfig) -> None:
        """Validate the configuration format."""
        required = ["build", "distributions"]
        missing = [key for key in required if key not in config]
        if missing:
            raise ConfigError(f"Missing required config sections: {', '.join(missing)}")

    def get_supported_distributions(self) -> list[str]:
        """Get list of supported distributions."""
        return list(self.config["distributions"]["dependencies"].keys())

    def generate_package_files(self, dist_name: str, metadata: PackageMetadata) -> None:
        """Generate package files for a specific distribution."""
        try:
            template = self.env.get_template(f"{dist_name}.jinja2")
        except jinja2.TemplateNotFound:
            raise TemplateError(f"Template not found for {dist_name}")

        output_path = self.get_output_path(dist_name, metadata)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        # Set directory permissions
        current_user = getpass.getuser()
        run_cmd(
            [
                "sudo",
                "chown",
                "-R",
                f"{current_user}:{current_user}",
                str(output_path.parent),
            ]
        )
        run_cmd(
            [
                "sudo",
                "chmod",
                "-R",
                "u+rw,g+rw",  # read/write for user and group
                str(output_path.parent),
            ]
        )

        try:
            content = template.render(
                metadata=metadata, config=self.config["distributions"]
            )
            output_path.write_text(content)
        except jinja2.TemplateError as e:
            raise TemplateError(f"Failed to render template: {e}") from e

    def get_output_path(self, dist_name: str, metadata: PackageMetadata) -> Path:
        """Get the output path for package files."""
        base_dir = self.build_dir / dist_name

        if dist_name.startswith("void-"):
            base_dir = base_dir / dist_name.split("-")[1]

        return base_dir / self._get_package_file_name(dist_name, metadata)

    def _get_package_file_name(self, dist_name: str, metadata: PackageMetadata) -> str:
        """Get the appropriate package file name for a distribution."""
        file_names = {
            "arch": "PKGBUILD",
            "deb": "control",
            "void-glibc": "template",
            "void-musl": "template",
        }
        return file_names.get(dist_name, "package.conf")

    def build_package(self, dist_name: str) -> Path:
        """Build package for a specific distribution."""
        metadata = PackageMetadata.from_cargo_toml(
            self.project_root / "Cargo.toml", self.config
        )

        builder = get_builder(dist_name, self.project_root)

        # Generate initial package files
        self.generate_package_files(dist_name, metadata)

        # Build and verify the package
        package_path = builder.build(metadata)
        builder.verify(package_path)

        return package_path


def main() -> NoReturn:
    """CLI entry point."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Generate and build packages")
    parser.add_argument(
        "--distributions",
        default="all",
        help="Comma-separated list of distributions to build for",
    )

    args = parser.parse_args()
    project_root = Path.cwd().parent.parent

    try:
        generator = PackageGenerator(project_root)

        if args.distributions == "all":
            distributions = generator.get_supported_distributions()
        else:
            distributions = args.distributions.split(",")

        for dist in distributions:
            print(f"\nBuilding package for {dist}...")
            package_path = generator.build_package(dist)
            print(f"Package built successfully: {package_path}")

        sys.exit(0)

    except (ConfigError, BuildError, ValidationError, CommandError, TemplateError) as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
