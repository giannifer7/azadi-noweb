# packaging/scripts/errors.py
"""Custom error types for the packaging system."""

class PackageError(Exception):
    """Base error for all packaging related errors."""

class BuildError(PackageError):
    """Error during package building."""

class ConfigError(PackageError):
    """Error in package configuration."""

class ValidationError(PackageError):
    """Error during package validation."""

class TemplateError(PackageError):
    """Error in template processing."""

class UnsupportedDistributionError(PackageError):
    """Error when a distribution is not supported."""
