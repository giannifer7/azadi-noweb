# Package Building Guide

## Local Development Setup

1. Install prerequisites:
   ```bash
   # Install Docker and Docker Compose
   sudo apt-get install docker.io docker-compose

   # Install Python 3.12
   sudo apt-get install python3.12 python3.12-venv
   ```

2. Create and activate virtual environment:
   ```bash
   python3.12 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

3. Generate package files:
   ```bash
   python packaging/scripts/generator.py --distributions=all
   ```

## Building Packages Locally

### Using Docker (Recommended)

1. Build Debian package:
   ```bash
   python packaging/scripts/docker.py debian
   ```

2. Build Void Linux packages:
   ```bash
   # For glibc
   python packaging/scripts/docker.py void-glibc

   # For musl
   python packaging/scripts/docker.py void-musl
   ```

3. Clean up:
   ```bash
   # Stop containers
   python packaging/scripts/docker.py down

   # Stop containers and remove volumes
   python packaging/scripts/docker.py clean
   ```

### Direct Building (for development)

1. Generate package files:
   ```bash
   python packaging/scripts/generator.py --distributions=arch
   ```

2. Build using local tools:
   ```bash
   cd packaging/build/arch
   makepkg -si
   ```

## Testing Built Packages

1. Test Debian package:
   ```bash
   sudo dpkg -i packaging/build/deb/*.deb
   azadi-noweb --version
   ```

2. Test Void package:
   ```bash
   sudo xbps-install -R hostdir/binpkgs azadi-noweb
   azadi-noweb --version
   ```

3. Test Arch package:
   ```bash
   sudo pacman -U packaging/build/arch/*.pkg.tar.zst
   azadi-noweb --version
   ```

## Common Issues

1. Missing dependencies:
   ```bash
   # Debian
   sudo apt-get install build-essential cargo

   # Void Linux
   sudo xbps-install base-devel rust cargo

   # Arch Linux
   sudo pacman -S base-devel rust cargo
   ```

2. Permission issues with Docker:
   ```bash
   # Add current user to docker group
   sudo usermod -aG docker $USER
   newgrp docker
   ```
