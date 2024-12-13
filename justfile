# Default recipe
default:
    @just --list

# Clean generated packages
clean-packages:
    python packaging/scripts/generate_packages.py --clean

# Generate packages
generate-packages:
    python packaging/scripts/generate_packages.py

# Generate packages without checksums
generate-packages-fast:
    python packaging/scripts/generate_packages.py --skip-checksums

# Generate specific package format(s)
generate FORMAT:
    python packaging/scripts/generate_packages.py --formats "{{FORMAT}}"

# Run package generator tests
