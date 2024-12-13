{ pkgs ? import <nixpkgs> {} }:

let
  pythonPackages = ps: with ps; [
    pip
    virtualenv
  ];
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    # Core build dependencies
    rustc
    cargo
    pkg-config

    # Development tools
    rust-analyzer
    clippy
    rustfmt
    just

    # Python environment
    (python312.withPackages pythonPackages)
    uv

    # Locale support
    glibcLocales
  ];

  # Set up locales
  LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";
  LANG = "en_US.UTF-8";
  LC_ALL = "en_US.UTF-8";

  shellHook = ''
    # Ensure the cache is configured
    if ! grep -q "azadi-noweb" ~/.config/nix/nix.conf 2>/dev/null; then
      echo "Configuring Cachix..."
      cachix use azadi-noweb
    fi

    # Install cargo tools if not present
    if ! command -v cargo-rpm >/dev/null 2>&1; then
      echo "Installing cargo-rpm..."
      cargo install cargo-rpm
    fi

    if ! command -v cargo-deb >/dev/null 2>&1; then
      echo "Installing cargo-deb..."
      cargo install cargo-deb
    fi

    # Setup Python environment if it doesn't exist
    if [ ! -d .venv ]; then
      echo "Creating Python virtual environment..."
      python -m venv .venv
    fi

    # Activate Python environment
    source .venv/bin/activate

    # Show available just commands
    if [ -f justfile ]; then
      echo "Available just commands:"
      just --list
    fi
  '';
}
