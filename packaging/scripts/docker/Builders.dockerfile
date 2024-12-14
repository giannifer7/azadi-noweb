# packaging/scripts/docker/Builders.dockerfile
# Base builder with common tools
FROM rust:1.75-slim as base
WORKDIR /build
RUN apt-get update && apt-get install -y \
    build-essential \
    python3 \
    python3-pip \
    git \
    && rm -rf /var/lib/apt/lists/*

# Debian builder with cargo-deb
FROM base as debian-builder
RUN cargo install cargo-deb
COPY packaging/scripts/pyproject.toml packaging/scripts/requirements.txt ./
RUN pip3 install -r requirements.txt
# We'll mount the project directory at runtime
CMD ["python3", "packaging/scripts/generator.py", "--distributions=deb"]

# Void Linux glibc builder
FROM ghcr.io/void-linux/void-linux:latest-full-x86_64 as void-glibc-builder
RUN xbps-install -Syu && \
    xbps-install -y bash git gcc rust cargo base-devel python3 python3-pip
COPY packaging/scripts/pyproject.toml packaging/scripts/requirements.txt ./
RUN pip3 install -r requirements.txt
CMD ["python3", "packaging/scripts/generator.py", "--distributions=void-glibc"]

# Void Linux musl builder
FROM ghcr.io/void-linux/void-linux:latest-full-x86_64-musl as void-musl-builder
RUN xbps-install -Syu && \
    xbps-install -y bash git gcc gcompat base-devel rust cargo python3 python3-pip
COPY packaging/scripts/pyproject.toml packaging/scripts/requirements.txt ./
RUN pip3 install -r requirements.txt
CMD ["python3", "packaging/scripts/generator.py", "--distributions=void-musl"]
