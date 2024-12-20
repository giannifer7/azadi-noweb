# src/pkg_gen/utils/command.py
from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


class CommandError(Exception):
    """Exception raised when a command execution fails."""

    def __init__(self, message: str):
        self.message = message
        super().__init__(self.message)


def run_cmd(
    cmd: list[str | Path], check: bool = True, **kwargs: Any
) -> subprocess.CompletedProcess:
    """
    Execute a command and return its output.

    Args:
        cmd: Command to execute as a list of arguments
        check: If True, raises CommandError on non-zero exit status
        **kwargs: Additional arguments passed to subprocess.run

    Returns:
        CompletedProcess instance with command results

    Raises:
        CommandError: If the command fails and check is True
    """
    try:
        cmd_str = [str(arg) for arg in cmd]
        return subprocess.run(
            cmd_str, check=check, capture_output=True, text=True, **kwargs
        )
    except subprocess.CalledProcessError as e:
        error_msg = f"""Command failed: {' '.join(str(x) for x in cmd)}
stdout: {e.stdout}
stderr: {e.stderr}
return code: {e.returncode}"""
        raise CommandError(error_msg)
