#!/usr/bin/env python3
"""Docker control script for Void Linux package building environments."""

import argparse
import os
import subprocess
import sys
from pathlib import Path

COMPOSE_YML = """version: '3.8'
services:
  void-glibc:
    image: ghcr.io/void-linux/void-linux:latest-full-x86_64
    volumes:
      - ../../..:/workspace
      - void-glibc-cargo:/root/.cargo
      - void-glibc-cache:/var/cache/xbps
    working_dir: /workspace/packaging/build/void/glibc
    command: sleep infinity
    init: true

  void-musl:
    image: ghcr.io/void-linux/void-linux:latest-full-x86_64-musl
    volumes:
      - ../../..:/workspace
      - void-musl-cargo:/root/.cargo
      - void-musl-cache:/var/cache/xbps
    working_dir: /workspace/packaging/build/void/musl
    command: sleep infinity
    init: true

volumes:
  void-glibc-cargo:
  void-glibc-cache:
  void-musl-cargo:
  void-musl-cache:
"""


def ensure_docker_compose():
    """Ensure docker compose file exists."""
    docker_dir = Path(__file__).parent / "docker"
    docker_dir.mkdir(exist_ok=True)
    compose_file = docker_dir / "docker-compose.yml"

    if not compose_file.exists():
        compose_file.write_text(COMPOSE_YML)

    return docker_dir


def run_command(cmd: list[str], check: bool = True) -> subprocess.CompletedProcess:
    """Run a command and handle errors."""
    try:
        return subprocess.run(cmd, check=check)
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(cmd)}")
        print(f"Exit code: {e.returncode}")
        sys.exit(e.returncode)


def start_container(variant: str):
    """Start and enter a container for the specified variant."""
    docker_dir = ensure_docker_compose()
    service = f"void-{variant}"

    # Start the container
    run_command(
        [
            "docker",
            "compose",
            "-f",
            str(docker_dir / "docker-compose.yml"),
            "up",
            "-d",
            service,
        ]
    )

    # Install dependencies and enter shell
    install_cmd = "xbps-install -Syu && xbps-install -y bash git gcc"
    if variant == "musl":
        install_cmd += " gcompat"
    install_cmd += " base-devel && bash"

    try:
        run_command(
            [
                "docker",
                "compose",
                "-f",
                str(docker_dir / "docker-compose.yml"),
                "exec",
                service,
                "sh",
                "-c",
                install_cmd,
            ]
        )
    except KeyboardInterrupt:
        print("\nExiting container...")


def stop_containers(clean: bool = False):
    """Stop containers and optionally remove volumes."""
    docker_dir = ensure_docker_compose()
    cmd = ["docker", "compose", "-f", str(docker_dir / "docker-compose.yml"), "down"]
    if clean:
        cmd.append("-v")
    run_command(cmd)


def main():
    parser = argparse.ArgumentParser(
        description="Manage Void Linux docker environments for package building"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    # glibc command
    subparsers.add_parser("glibc", help="Start and enter glibc container")

    # musl command
    subparsers.add_parser("musl", help="Start and enter musl container")

    # down command
    subparsers.add_parser("down", help="Stop containers")

    # clean command
    subparsers.add_parser("clean", help="Stop containers and remove volumes")

    args = parser.parse_args()

    if args.command in ["glibc", "musl"]:
        start_container(args.command)
    elif args.command == "down":
        stop_containers()
    elif args.command == "clean":
        stop_containers(clean=True)


if __name__ == "__main__":
    main()
