#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
import sys
import time
from pathlib import Path


def sha256sum(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha1sum(path: Path) -> str:
    return hashlib.sha1(path.read_bytes()).hexdigest()


def md5sum(path: Path) -> str:
    return hashlib.md5(path.read_bytes()).hexdigest()


def replace_file(path: Path, content: str) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(content)
    os.replace(tmp, path)


def wait_for_stable(paths: list[Path], timeout_s: float, min_mtime_ns: int) -> bool:
    deadline = time.monotonic() + timeout_s
    last_sig: tuple[tuple[int, int], ...] | None = None
    stable_polls = 0
    while time.monotonic() < deadline:
        if all(path.exists() for path in paths):
            stats = [path.stat() for path in paths]
            if any(stat.st_mtime_ns < min_mtime_ns for stat in stats):
                time.sleep(0.2)
                continue
            sig = tuple((stat.st_mtime_ns, stat.st_size) for stat in stats)
            if sig == last_sig:
                stable_polls += 1
                if stable_polls >= 5:
                    return True
            else:
                last_sig = sig
                stable_polls = 0
        time.sleep(0.2)
    return False


def normalize_buildinfo(path: Path) -> bool:
    text = path.read_text()
    lines = []
    changed = False
    for line in text.splitlines(keepends=True):
        if line.startswith(" DEB_BUILD_PROFILES=") and "noudeb" in line:
            changed = True
            continue
        if line.startswith("Built-For-Profiles:") and "noudeb" in line:
            changed = True
            continue
        lines.append(line)
    if changed:
        replace_file(path, "".join(lines))
    return changed


def normalize_changes(path: Path, buildinfo_name: str, buildinfo_path: Path) -> bool:
    text = path.read_text()
    size = buildinfo_path.stat().st_size
    sha1 = sha1sum(buildinfo_path)
    sha256 = sha256sum(buildinfo_path)
    md5 = md5sum(buildinfo_path)
    section = None
    changed = False
    out_lines: list[str] = []

    for line in text.splitlines(keepends=True):
        if line.startswith("Built-For-Profiles:") and "noudeb" in line:
            changed = True
            continue

        if line.startswith("Checksums-Sha1:"):
            section = "sha1"
            out_lines.append(line)
            continue
        if line.startswith("Checksums-Sha256:"):
            section = "sha256"
            out_lines.append(line)
            continue
        if line.startswith("Checksums-Md5:"):
            section = "md5"
            out_lines.append(line)
            continue
        if line.startswith("Files:"):
            section = "files"
            out_lines.append(line)
            continue
        if re.match(r"^[A-Z][A-Za-z0-9-]*:", line):
            section = None

        if line.rstrip("\n").endswith(f" {buildinfo_name}"):
            if section == "sha1":
                out_lines.append(f" {sha1} {size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "sha256":
                out_lines.append(f" {sha256} {size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "md5":
                out_lines.append(f" {md5} {size} {buildinfo_name}\n")
                changed = True
                continue
            if section == "files":
                match = re.match(r"^\s*[0-9a-f]+\s+\d+\s+(\S+\s+\S+)\s+" + re.escape(buildinfo_name) + r"\n?$", line)
                if match:
                    out_lines.append(f" {md5} {size} {match.group(1)} {buildinfo_name}\n")
                    changed = True
                    continue

        out_lines.append(line)

    if changed:
        replace_file(path, "".join(out_lines))
    return changed


def package_version(safe_dir: Path) -> str:
    return subprocess.check_output(
        ["dpkg-parsechangelog", "-l", str(safe_dir / "debian/changelog"), "-SVersion"],
        text=True,
    ).strip()


def host_arch() -> str:
    return subprocess.check_output(["dpkg-architecture", "-qDEB_HOST_ARCH"], text=True).strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=["binary", "source"], required=True)
    parser.add_argument("--wait-for-new", action="store_true")
    parser.add_argument("--timeout", type=float, default=900.0)
    args = parser.parse_args()

    script_dir = Path(__file__).resolve().parent
    safe_dir = script_dir.parent
    repo_root = safe_dir.parent
    started_ns = time.time_ns()
    version = package_version(safe_dir)
    arch = host_arch()

    if args.mode == "binary":
        buildinfo = repo_root / f"libpng1.6_{version}_{arch}.buildinfo"
        changes = repo_root / f"libpng1.6_{version}_{arch}.changes"
        watch_paths = [buildinfo, changes]
    else:
        buildinfo = repo_root / f"libpng1.6_{version}_source.buildinfo"
        changes = repo_root / f"libpng1.6_{version}_source.changes"
        dsc = repo_root / f"libpng1.6_{version}.dsc"
        debian_tar = repo_root / f"libpng1.6_{version}.debian.tar.xz"
        watch_paths = [buildinfo, changes, dsc, debian_tar]

    if args.wait_for_new:
        if not wait_for_stable(watch_paths, args.timeout, started_ns):
            return 0
    elif not all(path.exists() for path in watch_paths[:2]):
        return 0

    normalize_buildinfo(buildinfo)
    normalize_changes(changes, buildinfo.name, buildinfo)

    if args.mode == "source":
        subprocess.run(["bash", str(script_dir / "refresh-source-snapshot.sh")], check=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
