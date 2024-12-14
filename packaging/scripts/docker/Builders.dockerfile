# packaging/scripts/docker/Builders.dockerfile

# Base builder with minimal tools
FROM rust:1.75-slim as base
WORKDIR /build
RUN apt-get update && apt-get install -y \
    build-essential \
    python3 \
    python3-pip \
    git \
    && rm -rf /var/lib/apt/lists/*

# Debian builder
FROM base as debian-builder
RUN cargo install cargo-deb
COPY packaging/scripts/pyproject.toml packaging/scripts/requirements.txt ./
RUN pip3 install -r requirements.txt
CMD ["python3", "packaging/scripts/generate_packages.py", "--formats=deb"]

# Nix builder
FROM nixos/nix:latest as nix-builder
WORKDIR /build
# Install Python for our generator
RUN nix-env -iA nixpkgs.python3 nixpkgs.python3Packages.pip
COPY packaging/scripts/pyproject.toml packaging/scripts/requirements.txt ./
RUN pip3 install -r requirements.txt
CMD ["python3", "packaging/scripts/generate_packages.py", "--formats=nix"]
