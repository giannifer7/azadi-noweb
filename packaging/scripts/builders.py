# packaging/scripts/builders.py
"""Builders for different package formats."""

from __future__ import annotations

import subprocess
import shutil
from pathlib import Path
from typing import Protocol

from .errors import BuildError, ConfigError, ValidationError, UnsupportedDistributionError
from .metadata import PackageMetadata

class PackageBuilder(Protocol):
    """Interface for package builders."""
    
    @property
    def project_root(self) -> Path: ...
    
    @property
    def dist_name(self) -> str: ...
    
    @property
    def build_dir(self) -> Path: ...
    
    def build(self, metadata: PackageMetadata) -> Path:
        """
        Build the package.
        Raises BuildError on failure.
        """
        ...
        
    def verify(self, package_path: Path) -> None:
        """
        Verify the built package.
        Raises ValidationError if package is invalid.
        """
        ...

class DebianBuilder:
    """Builder for Debian packages using cargo-deb."""
    
    def __init__(self, project_root: Path, dist_name: str):
        self._project_root = project_root
        self._dist_name = dist_name
        self._build_dir = project_root / "packaging" / "build" / dist_name
    
    @property
    def project_root(self) -> Path:
        return self._project_root
        
    @property
    def dist_name(self) -> str:
        return self._dist_name
        
    @property
    def build_dir(self) -> Path:
        return self._build_dir
    
    def build(self, metadata: PackageMetadata) -> Path:
        config_path = self.build_dir / "Cargo.deb.toml"
        if not config_path.exists():
            raise ConfigError(f"Cargo.deb.toml not found at {config_path}")
            
        try:
            subprocess.run(
                ["cargo", "deb"],
                cwd=self.project_root,
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise BuildError(f"Failed to build Debian package: {e.stdout}") from e
        
        try:
            return next(self.build_dir.glob("*.deb"))
        except StopIteration:
            raise BuildError("No .deb package was generated")
        
    def verify(self, package_path: Path) -> None:
        try:
            subprocess.run(
                ["dpkg-deb", "--info", package_path],
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise ValidationError(f"Invalid Debian package: {e.stdout}")

class ArchBuilder:
    """Builder for Arch Linux packages using makepkg."""
    
    def __init__(self, project_root: Path, dist_name: str):
        self._project_root = project_root
        self._dist_name = dist_name
        self._build_dir = project_root / "packaging" / "build" / dist_name
    
    @property
    def project_root(self) -> Path:
        return self._project_root
        
    @property
    def dist_name(self) -> str:
        return self._dist_name
        
    @property
    def build_dir(self) -> Path:
        return self._build_dir
        
    def build(self, metadata: PackageMetadata) -> Path:
        pkgbuild_path = self.build_dir / "PKGBUILD"
        if not pkgbuild_path.exists():
            raise ConfigError(f"PKGBUILD not found at {pkgbuild_path}")
            
        try:
            # Build the package using makepkg
            subprocess.run(
                ["makepkg", "-f", "--noconfirm"],
                cwd=self.build_dir,
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise BuildError(f"Failed to build Arch package: {e.stdout}") from e
            
        try:
            return next(self.build_dir.glob(f"{metadata.package_name}-{metadata.version}*.pkg.tar.zst"))
        except StopIteration:
            raise BuildError("No .pkg.tar.zst package was generated")
    
    def verify(self, package_path: Path) -> None:
        try:
            subprocess.run(
                ["pacman", "-Qp", package_path],
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise ValidationError(f"Invalid Arch package: {e.stdout}")

class VoidBuilder:
    """Builder for Void Linux packages."""
    
    def __init__(self, project_root: Path, dist_name: str, libc_variant: str):
        self._project_root = project_root
        self._dist_name = f"void-{libc_variant}"
        self._build_dir = project_root / "packaging" / "build" / self._dist_name
        self.libc_variant = libc_variant
    
    @property
    def project_root(self) -> Path:
        return self._project_root
        
    @property
    def dist_name(self) -> str:
        return self._dist_name
        
    @property
    def build_dir(self) -> Path:
        return self._build_dir
        
    def build(self, metadata: PackageMetadata) -> Path:
        template_path = self.build_dir / "template"
        if not template_path.exists():
            raise ConfigError(f"void template not found at {template_path}")
            
        srcpkgs_dir = Path("/usr/src/void-packages/srcpkgs")
        package_dir = srcpkgs_dir / metadata.package_name
        
        try:
            package_dir.mkdir(exist_ok=True)
            shutil.copy2(template_path, package_dir / "template")
        except OSError as e:
            raise BuildError(f"Failed to set up Void package directory: {e}") from e
        
        try:
            subprocess.run(
                ["./xbps-src", "pkg", metadata.package_name],
                cwd="/usr/src/void-packages",
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise BuildError(f"Failed to build Void package: {e.stdout}") from e
        
        try:
            return next(Path("/usr/src/void-packages/hostdir/binpkgs").glob(
                f"{metadata.package_name}-*.xbps"
            ))
        except StopIteration:
            raise BuildError("No .xbps package was generated")
        
    def verify(self, package_path: Path) -> None:
        try:
            subprocess.run(
                ["xbps-rindex", "-v", package_path],
                check=True,
                capture_output=True,
                text=True
            )
        except subprocess.CalledProcessError as e:
            raise ValidationError(f"Invalid Void package: {e.stdout}")

def get_builder(dist_name: str, project_root: Path) -> PackageBuilder:
    """
    Factory function to get the appropriate builder.
    Raises UnsupportedDistributionError for unknown distributions.
    """
    if dist_name == "deb":
        return DebianBuilder(project_root, dist_name)
    elif dist_name.startswith("void-"):
        libc_variant = dist_name.split("-")[1]
        return VoidBuilder(project_root, dist_name, libc_variant)
    elif dist_name == "arch":
        return ArchBuilder(project_root, dist_name)
    else:
        raise UnsupportedDistributionError(f"Unsupported distribution: {dist_name}")
