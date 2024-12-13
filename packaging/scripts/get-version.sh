#!/bin/bash

CARGO_TOML_PATH="../../Cargo.toml"

# Check if the file exists
if [ ! -f "$CARGO_TOML_PATH" ]; then
  echo "Error: File not found at $CARGO_TOML_PATH"
  exit 1
fi

# Extract the version
VERSION=$(grep -E '^version\s*=\s*"[^"]+"' "$CARGO_TOML_PATH" | sed -E 's/version\s*=\s*"([^"]+)"/\1/')

# Check if a version was found
if [ -z "$VERSION" ]; then
  echo "Error: Could not extract version from $CARGO_TOML_PATH"
  exit 1
fi

# Output the extracted version
echo "$VERSION"
