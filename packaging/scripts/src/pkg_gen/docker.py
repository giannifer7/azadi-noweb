# packaging/scripts/docker.py
"""Docker environment management."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path
from typing import NoReturn

def run_command(cmd: list[str], check: bool = True) -> subprocess.CompletedProcess:
    """Run a command and handle errors."""
    try:
        return subprocess.run(cmd, check=check)
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {' '.join(cmd)}")
        print(f"Exit code: {e.returncode}")
        sys.exit(e.returncode)

def start_builder(variant: str, project_root: Path) -> None:
    """Start and enter a build container."""
    compose_file = project_root / "packaging" / "scripts" / "docker" / "docker-compose.yml"
    
    # Start the container
    run_command([
        "docker", "compose",
        "-f", str(compose_file),
        "run", "--rm",
        f"{variant}-build",
    ])

def cleanup(project_root: Path, remove_volumes: bool = False) -> None:
    """Stop containers and optionally remove volumes."""
    compose_file = project_root / "packaging" / "scripts" / "docker" / "docker-compose.yml"
    cmd = ["docker", "compose", "-f", str(compose_file), "down"]
    if remove_volumes:
        cmd.append("-v")
    run_command(cmd)

def main() -> NoReturn:
    parser = argparse.ArgumentParser(
        description="Manage Docker build environments"
    )
    parser.add_argument(
        "action",
        choices=["debian", "void-glibc", "void-musl", "down", "clean"],
        help="Action to perform"
    )
    
    args = parser.parse_args()
    project_root = Path(__file__).parent.parent.parent
    
    if args.action in ["debian", "void-glibc", "void-musl"]:
        start_builder(args.action, project_root)
    elif args.action == "down":
        cleanup(project_root)
    elif args.action == "clean":
        cleanup(project_root, remove_volumes=True)
    
    sys.exit(0)

if __name__ == "__main__":
    main()
