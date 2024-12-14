# packaging/scripts/generate_packages.py
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

def main():
    parser = argparse.ArgumentParser(description="Build packages for multiple distributions")
    parser.add_argument(
        "--distributions",
        default="all",
        help="Comma-separated list of distributions to build for"
    )
    
    args = parser.parse_args()
    project_root = Path(__file__).parent.parent.parent
    
    builder = PackageBuilder(project_root)
    
    if args.distributions == "all":
        distributions = builder.get_supported_distributions()
    else:
        distributions = set(args.distributions.split(","))
        unsupported = distributions - builder.get_supported_distributions()
        if unsupported:
            print(f"Warning: Unsupported distributions: {', '.join(unsupported)}")
            distributions -= unsupported
    
    for dist in sorted(distributions):
        print(f"\nBuilding package for {dist}...")
        try:
            builder.build_package(dist)
        except Exception as e:
            print(f"Error building package for {dist}: {e}")
            continue

if __name__ == "__main__":
    main()
# packaging/scripts/generate_packages.py
"""Package generation script for multiple distributions."""

import argparse
import hashlib
import re
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Any, Optional, Tuple, List

import jinja2

class PackageBuilder:
    """Manages package building across different distributions."""
    
    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.config = self._load_config()
        self.env = self._setup_jinja()
        
    def get_supported_distributions(self) -> List[str]:
        """Get list of supported distributions from config."""
        return list(self.config.get("distributions", {}).get("dependencies", {}).keys())
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
    def get_distribution_info(self, dist_name: str, dist_config: Dict[str, Any]) -> Dict[str, Any]:
        """Get distribution-specific information."""
        dist_info = dist_config.get("distributions", {})
        return {
            "architectures": dist_info.get("arch", ["x86_64"]),
            "dependencies": dist_info.get("dependencies", {}).get(dist_name, []),
            "libc_variant": dist_info.get("libc", {}).get(dist_name),
        }
    def build_package(self, distribution: str) -> None:
        """Build package for a specific distribution."""
        if distribution not in self.get_supported_distributions():
            raise ValueError(f"Unsupported distribution: {distribution}")
            
        cargo_toml = self.project_root / "Cargo.toml"
        metadata = PackageMetadata.from_cargo_toml(cargo_toml, self.config)
        
        if self.config.get("build", {}).get("compute_checksums", True):
            metadata.compute_checksums()
        
        self.generate_format(distribution, metadata)

    def _load_config(self) -> Dict[str, Any]:
        """Load distribution configuration from metadata.toml."""
        config_path = self.project_root / "packaging" / "config" / "metadata.toml"
        with open(config_path, "rb") as f:
            return tomllib.load(f)

    def _setup_jinja(self) -> jinja2.Environment:
        """Set up the Jinja2 environment for templates."""
        templates_dir = self.project_root / "packaging" / "templates"
        return jinja2.Environment(
            loader=jinja2.FileSystemLoader(templates_dir),
            undefined=jinja2.StrictUndefined,
            trim_blocks=True,
            lstrip_blocks=True,
        )

def main():
    """Main entry point for package generation."""
    parser = argparse.ArgumentParser(description="Build packages for multiple distributions")
    parser.add_argument(
        "--distributions",
        default="all",
        help="Comma-separated list of distributions to build for"
    )
    
    args = parser.parse_args()
    project_root = Path(__file__).parent.parent.parent
    
    builder = PackageBuilder(project_root)
    
    if args.distributions == "all":
        distributions = builder.get_supported_distributions()
    else:
        distributions = args.distributions.split(",")
    
    for dist in distributions:
        print(f"\nBuilding package for {dist}...")
        try:
            builder.build_package(dist)
        except Exception as e:
            print(f"Error building package for {dist}: {e}")
            continue

if __name__ == "__main__":
    main()
