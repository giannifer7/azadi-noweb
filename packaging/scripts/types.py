# packaging/scripts/types.py
"""Type definitions for the packaging system."""

from __future__ import annotations

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
