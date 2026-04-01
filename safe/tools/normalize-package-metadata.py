#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import hashlib
import os
import re
import subprocess
import sys
import time
from pathlib import Path


def md5sum(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def sha1sum(path: Path) -> str:
    return hashlib.sha1(path.read_bytes()).hexdigest()


def sha256sum(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_file(path: Path, content: str) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(content)
    os.replace(tmp, path)


def output_path(argv: list[str]) -> Path | None:
    for arg in argv:
        if arg == "-O":
            return None
        if arg.startswith("-O") and len(arg) > 2:
            return Path(arg[2:])
    return None


def stable_build_date() -> str | None:
    source_date_epoch = os.environ.get("SOURCE_DATE_EPOCH")
    if not source_date_epoch:
        return None
    epoch = int(source_date_epoch)
    tz = dt.datetime.now().astimezone().tzinfo
    return dt.datetime.fromtimestamp(epoch, tz=tz).strftime("%a, %d %b %Y %H:%M:%S %z")


def normalize_buildinfo(path: Path) -> bool:
    if not path.exists():
        return False

    build_date = stable_build_date()
    changed = False
    lines: list[str] = []

    for line in path.read_text().splitlines(keepends=True):
        if line.startswith("Build-Date:") and build_date is not None:
            normalized = f"Build-Date: {build_date}\n"
            if line != normalized:
                line = normalized
                changed = True
        if line.startswith(" DEB_BUILD_PROFILES=") and "noudeb" in line:
            line = ' DEB_BUILD_PROFILES=""\n'
            changed = True
        if line.startswith(" LC_ALL=") or line.startswith(" LC_CTYPE="):
            changed = True
            continue
        lines.append(line)

    if changed:
        replace_file(path, "".join(lines))
    return changed


def normalize_changes(path: Path, buildinfo_path: Path) -> bool:
    if not path.exists():
        return False

    buildinfo_name = buildinfo_path.name
    buildinfo_size = buildinfo_path.stat().st_size
    buildinfo_md5 = md5sum(buildinfo_path)
    buildinfo_sha1 = sha1sum(buildinfo_path)
    buildinfo_sha256 = sha256sum(buildinfo_path)

    changed = False
    section = None
    lines: list[str] = []

    for line in path.read_text().splitlines(keepends=True):
        if line.startswith("Built-For-Profiles:"):
            changed = True
            continue

        if line.startswith("Checksums-Sha1:"):
            section = "sha1"
            lines.append(line)
            continue
        if line.startswith("Checksums-Sha256:"):
            section = "sha256"
            lines.append(line)
            continue
        if line.startswith("Checksums-Md5:"):
            section = "md5"
            lines.append(line)
            continue
        if line.startswith("Files:"):
            section = "files"
            lines.append(line)
            continue
        if re.match(r"^[A-Z][A-Za-z0-9-]*:", line):
            section = None

        if line.rstrip("\n").endswith(f" {buildinfo_name}"):
            if section == "sha1":
                lines.append(f" {buildinfo_sha1} {buildinfo_size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "sha256":
                lines.append(f" {buildinfo_sha256} {buildinfo_size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "md5":
                lines.append(f" {buildinfo_md5} {buildinfo_size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "files":
                match = re.match(
                    r"^\s*[0-9a-f]+\s+\d+\s+(\S+\s+\S+)\s+" + re.escape(buildinfo_name) + r"\n?$",
                    line,
                )
                if match:
                    lines.append(f" {buildinfo_md5} {buildinfo_size} {match.group(1)} {buildinfo_name}\n")
                    changed = True
                    continue

        lines.append(line)

    if changed:
        replace_file(path, "".join(lines))
    return changed


def path_ready(path: Path, start_mtime_ns: int, wait_seconds: float = 0.2) -> bool:
    if not path.exists():
        return False
    first_stat = path.stat()
    if first_stat.st_mtime_ns < start_mtime_ns:
        return False
    first = first_stat.st_size
    time.sleep(wait_seconds)
    if not path.exists():
        return False
    second_stat = path.stat()
    return second_stat.st_mtime_ns >= start_mtime_ns and second_stat.st_size == first


def watch_mode(target: str) -> int:
    script_dir = Path(__file__).resolve().parent
    safe_dir = script_dir.parent
    repo_root = safe_dir.parent
    package_version = subprocess.check_output(
        ["dpkg-parsechangelog", "-l", str(safe_dir / "debian/changelog"), "-SVersion"],
        text=True,
    ).strip()
    host_arch = subprocess.check_output(["dpkg-architecture", "-qDEB_HOST_ARCH"], text=True).strip()

    if target == "binary":
        buildinfo_path = repo_root / f"libpng1.6_{package_version}_{host_arch}.buildinfo"
        changes_path = repo_root / f"libpng1.6_{package_version}_{host_arch}.changes"
        timeout = 120.0
    elif target == "source":
        buildinfo_path = repo_root / f"libpng1.6_{package_version}_source.buildinfo"
        changes_path = repo_root / f"libpng1.6_{package_version}_source.changes"
        timeout = 120.0
    else:
        print(f"unknown watch target: {target}", file=sys.stderr)
        return 2

    start_mtime_ns = time.time_ns()
    deadline = time.monotonic() + timeout
    buildinfo_normalized = False

    while time.monotonic() < deadline:
        if not buildinfo_normalized and path_ready(buildinfo_path, start_mtime_ns):
            normalize_buildinfo(buildinfo_path)
            buildinfo_normalized = True

        if buildinfo_normalized and path_ready(changes_path, start_mtime_ns):
            normalize_changes(changes_path, buildinfo_path)
            if target == "source":
                subprocess.run(["bash", str(script_dir / "refresh-source-snapshot.sh")], check=True)
            files_list = safe_dir / "debian/files"
            if files_list.exists():
                files_list.unlink()
            return 0

        time.sleep(0.2)

    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print(
            "usage: normalize-package-metadata.py <buildinfo|changes> <tool-args...>\n"
            "   or: normalize-package-metadata.py watch <binary|source>",
            file=sys.stderr,
        )
        return 2

    if sys.argv[1] == "watch":
        if len(sys.argv) != 3:
            print("usage: normalize-package-metadata.py watch <binary|source>", file=sys.stderr)
            return 2
        return watch_mode(sys.argv[2])

    if sys.argv[1] not in {"buildinfo", "changes"}:
        print("usage: normalize-package-metadata.py <buildinfo|changes> <tool-args...>", file=sys.stderr)
        return 2

    mode = sys.argv[1]
    out_path = output_path(sys.argv[2:])
    if out_path is None:
        return 0

    if mode == "buildinfo":
        normalize_buildinfo(out_path)
        return 0

    buildinfo_path = out_path.with_suffix(".buildinfo")
    if buildinfo_path.exists():
        normalize_changes(out_path, buildinfo_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
