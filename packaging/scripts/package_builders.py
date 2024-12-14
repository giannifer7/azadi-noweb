# packaging/scripts/package_builders.py
"""Package builders for different distributions."""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path
from typing import Protocol

from generate_packages import PackageMetadata

class PackageBuilder:
    """Base class for package builders."""
    
    def __init__(self, project_root: Path, dist_name: str):
        self.project_root = project_root
        self.dist_name = dist_name
        self.build_dir = project_root / "packaging" / "build" / dist_name
        
    def build(self, metadata: PackageMetadata) -> Path:
        """Build the package. Must be implemented by subclasses."""
        raise NotImplementedError
        
    def verify(self, package_path: Path) -> bool:
        """Verify the built package. Must be implemented by subclasses."""
        raise NotImplementedError

class DebianBuilder(PackageBuilder):
    """Builder for Debian packages using cargo-deb."""
    
    def build(self, metadata: PackageMetadata) -> Path:
        # Ensure cargo-deb config exists
        config_path = self.build_dir / "Cargo.deb.toml"
        if not config_path.exists():
            raise FileNotFoundError("Cargo.deb.toml not found")
            
        # Build using cargo-deb
        subprocess.run(
            ["cargo", "deb"],
            cwd=self.project_root,
            check=True,
        )
        
        # Find the generated .deb file
        deb_path = next(self.build_dir.glob("*.deb"))
        return deb_path
        
    def verify(self, package_path: Path) -> bool:
        result = subprocess.run(
            ["dpkg-deb", "--info", package_path],
            capture_output=True,
            text=True,
        )
        return result.returncode == 0

class VoidBuilder(PackageBuilder):
    """Builder for Void Linux packages."""
    
    def __init__(self, project_root: Path, libc_variant: str):
        super().__init__(project_root, f"void-{libc_variant}")
        self.libc_variant = libc_variant
        
    def build(self, metadata: PackageMetadata) -> Path:
        template_path = self.build_dir / "template"
        if not template_path.exists():
            raise FileNotFoundError("void template not found")
            
        # Build using xbps-src
        srcpkgs_dir = Path("/usr/src/void-packages/srcpkgs")
        package_dir = srcpkgs_dir / metadata.package_name
        package_dir.mkdir(exist_ok=True)
        
        shutil.copy2(template_path, package_dir / "template")
        
        subprocess.run(
            ["./xbps-src", "pkg", metadata.package_name],
            cwd="/usr/src/void-packages",
            check=True,
        )
        
        # Find the generated package
        hostdir = Path("/usr/src/void-packages/hostdir/binpkgs")
        package_path = next(hostdir.glob(f"{metadata.package_name}-*.xbps"))
        return package_path
        
    def verify(self, package_path: Path) -> bool:
        result = subprocess.run(
            ["xbps-rindex", "-v", package_path],
            capture_output=True,
            text=True,
        )
        return result.returncode == 0

def get_builder(dist_name: str, project_root: Path) -> PackageBuilder:
    """Factory function to get the appropriate builder."""
    if dist_name == "deb":
        return DebianBuilder(project_root, dist_name)
    elif dist_name.startswith("void-"):
        libc_variant = dist_name.split("-")[1]
        return VoidBuilder(project_root, libc_variant)
    else:
        raise ValueError(f"Unsupported distribution: {dist_name}")
