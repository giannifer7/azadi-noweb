# List all recipes
default:
    @just --list

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt --check

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Build in release mode
build:
    cargo build --release

# Create all packages
package: (rpm) (deb)
    @echo "All packages created"

# Create RPM package
rpm:
    cargo rpm build

# Create DEB package
deb:
    cargo deb

# Clean all build artifacts
clean:
    cargo clean
    rm -rf target/

# Update dependencies
update:
    cargo update

# Development build with file watching
watch:
    cargo watch -x check -x test

# Install all required cargo tools
setup:
    cargo install cargo-rpm cargo-deb cargo-watch

# Run basic checks before commit
pre-commit: fmt lint test
    @echo "All pre-commit checks passed!"

# Generate and upload packages to cache
cache: build
    nix-build
    cachix push azadi-noweb result
