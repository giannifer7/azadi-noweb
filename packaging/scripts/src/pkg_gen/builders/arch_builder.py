from __future__ import annotations

import os
import hashlib
import re
import tempfile
from pathlib import Path
from contextlib import ExitStack

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from pkg_gen.errors import BuildError, ConfigError, ValidationError
from pkg_gen.metadata import PackageMetadata
from pkg_gen.utils.command import run_cmd, CommandError


class ArchBuilder:
    def __init__(self, project_root: Path, dist_name: str):
        self._project_root = project_root
        self._dist_name = dist_name
        self._build_dir = project_root / "packaging" / "build" / dist_name
        self._output_dir = project_root / "packaging" / "dist" / dist_name
        self._chroot_path = project_root / "packaging" / "build" / "chroot"

        retry_strategy = Retry(
            total=5, backoff_factor=1, status_forcelist=[429, 500, 502, 503, 504]
        )
        self._session = requests.Session()
        self._session.mount("https://", HTTPAdapter(max_retries=retry_strategy))

    @property
    def project_root(self) -> Path:
        return self._project_root

    @property
    def dist_name(self) -> str:
        return self._dist_name

    @property
    def build_dir(self) -> Path:
        return self._build_dir

    def _ensure_devtools_installed(self) -> None:
        try:
            run_cmd(["pacman", "-Qi", "devtools"])
        except CommandError:
            raise BuildError(
                "devtools is not installed. Please install it with: sudo pacman -S devtools"
            )

    def _process_makepkg_line(self, line: str, settings: dict[str, str]) -> str:
        if re.match(r"^(#)?PACKAGER=", line.strip()) and "PACKAGER" in settings:
            return f'PACKAGER="{settings["PACKAGER"]}"\n'
        return line

    def _update_makepkg_conf(self, makepkg_conf: Path) -> None:
        if not makepkg_conf.exists():
            return

        settings = {}
        try:
            with open("/etc/makepkg.conf") as f:
                for line in f:
                    line = line.strip()
                    if not line.startswith("#") and line.startswith("PACKAGER="):
                        settings["PACKAGER"] = line.split("=", 1)[1].strip().strip('"')
        except (OSError, IOError) as e:
            raise BuildError(f"Failed to read host makepkg.conf: {e}")

        with ExitStack() as stack:
            tmp = stack.enter_context(
                tempfile.NamedTemporaryFile(mode="w", delete=False)
            )
            tmp_path = Path(tmp.name)
            stack.callback(tmp_path.unlink)

            with open(makepkg_conf) as f:
                new_lines = [self._process_makepkg_line(line, settings) for line in f]

            tmp.write("".join(new_lines))
            tmp.flush()

            run_cmd(["sudo", "install", "-m", "644", tmp_path, makepkg_conf])

    def _download_release(self, metadata: PackageMetadata) -> tuple[Path, str]:
        url = f"{metadata.repo_url}/archive/refs/tags/v{metadata.version}.tar.gz"
        response = self._session.get(url, timeout=30)
        response.raise_for_status()
        sha512 = hashlib.sha512()
        sha512.update(response.content)
        checksum = sha512.hexdigest()
        tarball = self.build_dir / f"{metadata.package_name}-{metadata.version}.tar.gz"
        tarball.write_bytes(response.content)
        return tarball, checksum

    def _prepare_build(self, metadata: PackageMetadata) -> None:
        self.build_dir.mkdir(parents=True, exist_ok=True)
        pkgbuild_path = self.build_dir / "PKGBUILD"
        if not pkgbuild_path.exists():
            raise ConfigError(f"PKGBUILD not found at {pkgbuild_path}")

        _, checksum = self._download_release(metadata)
        content = pkgbuild_path.read_text()
        content = content.replace("sha512sums=('None')", f"sha512sums=('{checksum}')")
        pkgbuild_path.write_text(content)

        if not pkgbuild_path.exists():
            raise BuildError("PKGBUILD file missing after preparation")
        if not list(self.build_dir.glob("*.tar.gz")):
            raise BuildError("Source tarball missing after preparation")

    def _sign_package(self, package_path: Path) -> None:
        try:
            run_cmd(["gpg", "--detach-sign", str(package_path)])
            # Force a sync to ensure filesystem catches up
            run_cmd(["sync"])

            sig_path = Path(str(package_path) + ".sig")
            if not sig_path.exists():
                raise BuildError(f"Signature file not found at {sig_path}")
        except CommandError as e:
            raise BuildError(f"Failed to sign package {package_path.name}: {e}")

    def _extract_buildinfo(self, package_path: Path) -> None:
        try:
            run_cmd(
                [
                    "tar",
                    "-C",
                    str(package_path.parent),
                    "-xf",
                    str(package_path),
                    ".BUILDINFO",
                ]
            )
        except CommandError as e:
            raise BuildError(
                f"Failed to extract .BUILDINFO from {package_path.name}: {e}"
            )

    def _collect_outputs(self, metadata: PackageMetadata) -> list[Path]:
        self._output_dir.mkdir(parents=True, exist_ok=True)

        outputs = []
        for pkg in self.build_dir.glob(
            f"{metadata.package_name}*-{metadata.version}*.pkg.tar.zst"
        ):
            dest = self._output_dir / pkg.name
            pkg.rename(dest)
            outputs.append(dest)
            self._sign_package(dest)
            self._extract_buildinfo(dest)

            buildinfo_path = dest.parent / ".BUILDINFO"
            if buildinfo_path.exists():
                new_buildinfo = Path(str(dest) + ".buildinfo")
                buildinfo_path.rename(new_buildinfo)
                outputs.append(new_buildinfo)

        if not outputs:
            raise BuildError("No packages were generated")

        return outputs

    def _cleanup_chroot(self) -> None:
        root_path = self._chroot_path / "root"
        if not root_path.exists():
            return

        try:
            if os.path.ismount(str(root_path)):
                run_cmd(["sudo", "umount", "-R", str(root_path)])
        except CommandError:
            # Ignore umount errors, proceed with removal
            pass

        if root_path.is_dir():
            run_cmd(["sudo", "rm", "-rf", str(root_path)])

        lock_file = self._chroot_path / "root.lock"
        if lock_file.exists():
            lock_file.unlink()

    def build(self, metadata: PackageMetadata) -> list[Path]:
        self._ensure_devtools_installed()
        self._prepare_build(metadata)

        env = os.environ.copy()
        env["CHROOT"] = str(self._chroot_path)

        self._cleanup_chroot()

        try:
            run_cmd(
                ["mkarchroot", str(self._chroot_path / "root"), "base-devel"],
                env=env,
            )
        except CommandError as e:
            raise BuildError(f"Failed to create chroot: {e}")

        # Update makepkg.conf for PACKAGER setting
        makepkg_conf = self._chroot_path / "root" / "etc" / "makepkg.conf"
        self._update_makepkg_conf(makepkg_conf)

        try:
            run_cmd(
                ["extra-x86_64-build", "--"],
                cwd=self.build_dir,
                env=env,
            )
        except CommandError as e:
            raise BuildError(f"Package build failed: {e}")

        return self._collect_outputs(metadata)

    def verify(self, package_paths: Path | list[Path]) -> None:
        if isinstance(package_paths, Path):
            paths = [package_paths]
        else:
            paths = package_paths

        for package_path in paths:
            if not str(package_path).endswith(".pkg.tar.zst"):
                continue

            try:
                run_cmd(["pacman", "-Qp", str(package_path)])
            except CommandError as e:
                raise ValidationError(
                    f"Package verification failed for {package_path.name}: {e}"
                )

            sig_path = Path(str(package_path) + ".sig")
            if not sig_path.exists():
                raise ValidationError(
                    f"Package signature is missing for {package_path.name}"
                )

            try:
                run_cmd(["gpg", "--verify", str(sig_path), str(package_path)])
            except CommandError as e:
                raise ValidationError(
                    f"Package signature verification failed for {package_path.name}: {e}"
                )

            buildinfo_path = Path(str(package_path) + ".buildinfo")
            if not buildinfo_path.exists():
                raise ValidationError(
                    f"Missing .BUILDINFO file for {package_path.name}"
                )

            buildinfo_content = buildinfo_path.read_text()
            required_fields = [
                "format",
                "pkgname",
                "pkgver",
                "pkgarch",
                "packager",
                "builddate",
            ]
            missing = [
                field
                for field in required_fields
                if not any(
                    line.startswith(f"{field} = ")
                    for line in buildinfo_content.splitlines()
                )
            ]
            if missing:
                raise ValidationError(
                    f"Missing required fields in .BUILDINFO for {package_path.name}: {', '.join(missing)}"
                )

    def cleanup(self) -> None:
        if self.build_dir.exists():
            try:
                run_cmd(["rm", "-rf", str(self.build_dir)])
            except CommandError as e:
                print(f"Warning: Failed to clean up build directory: {e}")
