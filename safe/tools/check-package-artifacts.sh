#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
host_arch="$(dpkg-architecture -qDEB_HOST_ARCH)"

latest_matching_artifact() {
  local pattern="$1"

  find "$repo_root" -maxdepth 1 -type f -name "$pattern" -printf '%T@ %p\n' \
    | sort -nr \
    | head -n1 \
    | cut -d' ' -f2-
}

latest_artifact() {
  local package_name="$1"
  local extension="$2"

  latest_matching_artifact "${package_name}_*.${extension}"
}

latest_arch_artifact() {
  local package_name="$1"
  local extension="$2"

  latest_matching_artifact "${package_name}_*_${host_arch}.${extension}"
}

latest_source_split_artifact() {
  local package_name="$1"
  local extension="$2"

  latest_matching_artifact "${package_name}_*_source.${extension}"
}

require_artifact() {
  local package_name="$1"
  local extension="$2"
  local artifact

  artifact="$(latest_artifact "$package_name" "$extension")"
  if [[ -z "$artifact" ]]; then
    printf 'missing built package artifact: %s_*.%s\n' "$package_name" "$extension" >&2
    exit 1
  fi

  printf '%s\n' "$artifact"
}

require_arch_artifact() {
  local package_name="$1"
  local extension="$2"
  local artifact

  artifact="$(latest_arch_artifact "$package_name" "$extension")"
  if [[ -z "$artifact" ]]; then
    printf 'missing built package artifact: %s_*_%s.%s\n' \
      "$package_name" "$host_arch" "$extension" >&2
    exit 1
  fi

  printf '%s\n' "$artifact"
}

require_source_orig_tar() {
  local package_name="$1"
  local artifact

  artifact="$(latest_matching_artifact "${package_name}_*.orig.tar.xz")"
  if [[ -z "$artifact" ]]; then
    printf 'missing built source orig tar artifact: %s_*.orig.tar.xz\n' "$package_name" >&2
    exit 1
  fi

  printf '%s\n' "$artifact"
}

require_safe_source_snapshot_tar() {
  local package_name="$1"
  local artifact

  artifact="$(
    find "$repo_root" -maxdepth 1 -type f \
      -name "${package_name}_*.tar.xz" \
      ! -name "${package_name}_*.debian.tar.xz" \
      ! -name "${package_name}_*.orig.tar.xz" \
      -printf '%T@ %p\n' \
      | sort -nr \
      | head -n1 \
      | cut -d' ' -f2-
  )"
  if [[ -z "$artifact" ]]; then
    printf 'missing refreshed safe source snapshot artifact: %s_*.tar.xz\n' "$package_name" >&2
    exit 1
  fi

  printf '%s\n' "$artifact"
}

require_path() {
  local root="$1"
  local rel="$2"

  if [[ ! -e "$root/$rel" && ! -L "$root/$rel" ]]; then
    printf 'missing packaged path: %s\n' "$rel" >&2
    exit 1
  fi
}

require_symlink_target() {
  local root="$1"
  local rel="$2"
  local expected="$3"

  require_path "$root" "$rel"
  if [[ "$(readlink "$root/$rel")" != "$expected" ]]; then
    printf 'unexpected symlink target for %s: expected %s, found %s\n' \
      "$rel" "$expected" "$(readlink "$root/$rel")" >&2
    exit 1
  fi
}

require_changes_entry() {
  local changes_file="$1"
  local artifact="$2"
  local basename_artifact

  basename_artifact="$(basename "$artifact")"
  if ! grep -Fq " $basename_artifact" "$changes_file"; then
    printf 'changes file %s does not reference %s\n' "$changes_file" "$basename_artifact" >&2
    exit 1
  fi
}

require_buildinfo_entry() {
  local buildinfo_file="$1"
  local artifact="$2"
  local basename_artifact

  basename_artifact="$(basename "$artifact")"
  if ! grep -Fq " $basename_artifact" "$buildinfo_file"; then
    printf 'buildinfo file %s does not reference %s\n' "$buildinfo_file" "$basename_artifact" >&2
    exit 1
  fi
}

require_buildinfo_metadata() {
  local buildinfo_file="$1"
  local expected_version="$2"

  if ! grep -Fq "Source: libpng1.6" "$buildinfo_file"; then
    printf 'unexpected source stanza in %s\n' "$buildinfo_file" >&2
    exit 1
  fi
  if ! grep -Fq "Version: $expected_version" "$buildinfo_file"; then
    printf 'buildinfo version mismatch in %s\n' "$buildinfo_file" >&2
    exit 1
  fi
}

require_udeb_metadata_contract() {
  if awk '
    $1 == "Package:" { in_udeb = ($2 == "libpng16-16-udeb") }
    in_udeb && $1 == "Build-Profiles:" { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "$safe_dir/debian/control"; then
    printf 'libpng16-16-udeb still declares a Build-Profiles gate in %s\n' \
      "$safe_dir/debian/control" >&2
    exit 1
  fi
}

require_no_noudeb_profile_metadata() {
  local artifact="$1"

  if [[ -f "$artifact" ]] && grep -Eq '^Built-For-Profiles:.*(^|[[:space:],])noudeb([[:space:],]|$)' "$artifact"; then
    printf 'artifact %s advertises Built-For-Profiles: noudeb even though the refreshed packaging contract must ship libpng16-16-udeb\n' \
      "$artifact" >&2
    exit 1
  fi
}

require_buildinfo_environment_excludes_noudeb() {
  local artifact="$1"

  if [[ -f "$artifact" ]] && grep -Eq '^ DEB_BUILD_PROFILES=".*(^|[[:space:],])noudeb([[:space:],]|$)' "$artifact"; then
    printf 'artifact %s still records DEB_BUILD_PROFILES=noudeb in its build environment\n' "$artifact" >&2
    exit 1
  fi
}

artifacts_need_postbuild_settle() {
  if grep -Eq '^Built-For-Profiles:.*(^|[[:space:],])noudeb([[:space:],]|$)' "$binary_changes_artifact"; then
    return 0
  fi
  if [[ -n "$source_changes_artifact" ]] && grep -Eq '^Built-For-Profiles:.*(^|[[:space:],])noudeb([[:space:],]|$)' "$source_changes_artifact"; then
    return 0
  fi
  if grep -Eq '^ DEB_BUILD_PROFILES=".*(^|[[:space:],])noudeb([[:space:],]|$)' "$binary_buildinfo_artifact"; then
    return 0
  fi
  if [[ -n "$source_buildinfo_artifact" ]] && grep -Eq '^ DEB_BUILD_PROFILES=".*(^|[[:space:],])noudeb([[:space:],]|$)' "$source_buildinfo_artifact"; then
    return 0
  fi
  if ! validate_safe_snapshot_tar_matches_tree "$safe_source_snapshot_tar" >/dev/null 2>&1; then
    return 0
  fi

  return 1
}

wait_for_postbuild_settle() {
  local deadline=$((SECONDS + 15))

  while (( SECONDS < deadline )); do
    if ! artifacts_need_postbuild_settle; then
      return 0
    fi
    sleep 0.2
  done
}

validate_source_package_matches_tree() {
  local source_root="$1"

  python3 - <<'PY' "$repo_root" "$source_root"
import pathlib
import subprocess
import sys

repo_root = pathlib.Path(sys.argv[1])
source_root = pathlib.Path(sys.argv[2])

tracked = subprocess.check_output(
    [
        "git",
        "-C",
        str(repo_root),
        "ls-files",
        "--",
        "safe/Cargo.toml",
        "safe/build.rs",
        "safe/UNSAFE.md",
        "safe/abi",
        "safe/cshim",
        "safe/debian",
        "safe/include",
        "safe/node_modules",
        "safe/pkg",
        "safe/src",
        "safe/tools",
    ],
    text=True,
).splitlines()
tracked = [
    path
    for path in tracked
    if (repo_root / path).exists() or (repo_root / path).is_symlink()
]

tracked_rel = [path[len("safe/") :] for path in tracked]
source_files = set()
for rel in [
    "Cargo.toml",
    "build.rs",
    "UNSAFE.md",
    "abi",
    "cshim",
    "debian",
    "include",
    "node_modules",
    "pkg",
    "src",
    "tools",
]:
    base = source_root / rel
    if base.is_file() or base.is_symlink():
        source_files.add(rel)
        continue
    if not base.exists():
        continue
    for path in sorted(base.rglob("*")):
        if path.is_file() or path.is_symlink():
            source_files.add(str(path.relative_to(source_root)))

tracked_set = set(tracked_rel)
missing = sorted(tracked_set - source_files)
extra = sorted(source_files - tracked_set)
if missing:
    raise SystemExit(
        "source package is missing tracked package-affecting paths: "
        + ", ".join(missing)
    )
if extra:
    raise SystemExit(
        "source package contains unexpected package-affecting paths: "
        + ", ".join(extra)
    )

for rel in tracked_rel:
    repo_path = repo_root / "safe" / rel
    source_path = source_root / rel

    if repo_path.is_symlink() != source_path.is_symlink():
        raise SystemExit(f"path type mismatch for {rel}")

    if repo_path.is_symlink():
        if repo_path.readlink() != source_path.readlink():
            raise SystemExit(f"symlink target mismatch for {rel}")
        continue

    if repo_path.read_bytes() != source_path.read_bytes():
        raise SystemExit(f"source package content mismatch for {rel}")
PY
}

validate_safe_snapshot_tar_matches_tree() {
  local snapshot_tar="$1"

  python3 - <<'PY' "$repo_root" "$snapshot_tar"
import pathlib
import subprocess
import sys
import tarfile

repo_root = pathlib.Path(sys.argv[1])
snapshot_tar = pathlib.Path(sys.argv[2])

tracked = subprocess.check_output(
    ["git", "-C", str(repo_root), "ls-files", "--", "safe"],
    text=True,
).splitlines()
tracked = [
    path
    for path in tracked
    if (repo_root / path).exists() or (repo_root / path).is_symlink()
]

with tarfile.open(snapshot_tar, "r:*") as tf:
    members = {
        member.name: member
        for member in tf.getmembers()
        if member.isfile() or member.issym()
    }

tracked_set = set(tracked)
member_set = set(members)
missing = sorted(tracked_set - member_set)
extra = sorted(member_set - tracked_set)
if missing:
    raise SystemExit(
        "safe source snapshot tar is missing tracked paths: "
        + ", ".join(missing)
    )
if extra:
    raise SystemExit(
        "safe source snapshot tar contains unexpected paths: "
        + ", ".join(extra)
    )

with tarfile.open(snapshot_tar, "r:*") as tf:
    for rel in tracked:
        repo_path = repo_root / rel
        member = members[rel]

        if repo_path.is_symlink() != member.issym():
            raise SystemExit(f"path type mismatch for {rel}")

        if repo_path.is_symlink():
            if repo_path.readlink().as_posix() != member.linkname:
                raise SystemExit(f"symlink target mismatch for {rel}")
            continue

        extracted = tf.extractfile(member)
        if extracted is None:
            raise SystemExit(f"unable to read archived file {rel}")
        if repo_path.read_bytes() != extracted.read():
            raise SystemExit(f"safe source snapshot content mismatch for {rel}")
PY
}

runtime_deb="$(require_artifact libpng16-16t64 deb)"
dev_deb="$(require_artifact libpng-dev deb)"
tools_deb="$(require_artifact libpng-tools deb)"
udeb_artifact="$(require_artifact libpng16-16-udeb udeb)"
binary_changes_artifact="$(require_arch_artifact libpng1.6 changes)"
binary_buildinfo_artifact="$(require_arch_artifact libpng1.6 buildinfo)"
source_changes_artifact="$(latest_source_split_artifact libpng1.6 changes)"
source_buildinfo_artifact="$(latest_source_split_artifact libpng1.6 buildinfo)"
source_dsc="$(require_artifact libpng1.6 dsc)"
source_debian_tar="$(require_artifact libpng1.6 debian.tar.xz)"
source_orig_tar="$(require_source_orig_tar libpng1.6)"
safe_source_snapshot_tar="$(require_safe_source_snapshot_tar libpng1.6)"

wait_for_postbuild_settle

require_udeb_metadata_contract

if [[ -n "$source_changes_artifact" && -z "$source_buildinfo_artifact" ]]; then
  printf 'missing source buildinfo artifact for %s\n' "$source_changes_artifact" >&2
  exit 1
fi
if [[ -z "$source_changes_artifact" && -n "$source_buildinfo_artifact" ]]; then
  printf 'missing source changes artifact for %s\n' "$source_buildinfo_artifact" >&2
  exit 1
fi

require_no_noudeb_profile_metadata "$binary_changes_artifact"
require_no_noudeb_profile_metadata "$source_changes_artifact"
require_buildinfo_environment_excludes_noudeb "$binary_buildinfo_artifact"
require_buildinfo_environment_excludes_noudeb "$source_buildinfo_artifact"

for pair in \
  "libpng16-16t64:$runtime_deb" \
  "libpng-dev:$dev_deb" \
  "libpng-tools:$tools_deb" \
  "libpng16-16-udeb:$udeb_artifact"
do
  expected_name="${pair%%:*}"
  artifact="${pair#*:}"
  actual_name="$(dpkg-deb -f "$artifact" Package)"

  if [[ "$actual_name" != "$expected_name" ]]; then
    printf 'unexpected package name in %s: expected %s, found %s\n' \
      "$artifact" "$expected_name" "$actual_name" >&2
    exit 1
  fi
done

runtime_version="$(dpkg-deb -f "$runtime_deb" Version)"
for artifact in "$dev_deb" "$tools_deb" "$udeb_artifact"; do
  if [[ "$(dpkg-deb -f "$artifact" Version)" != "$runtime_version" ]]; then
    printf 'package version mismatch: expected %s in %s\n' "$runtime_version" "$artifact" >&2
    exit 1
  fi
done

for binary_artifact in \
  "$runtime_deb" \
  "$dev_deb" \
  "$tools_deb" \
  "$udeb_artifact" \
  "$binary_buildinfo_artifact"
do
  require_changes_entry "$binary_changes_artifact" "$binary_artifact"
done

require_buildinfo_metadata "$binary_buildinfo_artifact" "$runtime_version"
for artifact in "$runtime_deb" "$dev_deb" "$tools_deb" "$udeb_artifact"; do
  require_buildinfo_entry "$binary_buildinfo_artifact" "$artifact"
done

if [[ -n "$source_changes_artifact" ]]; then
  for source_artifact in \
    "$source_dsc" \
    "$source_orig_tar" \
    "$source_debian_tar" \
    "$source_buildinfo_artifact"
  do
    require_changes_entry "$source_changes_artifact" "$source_artifact"
  done

  require_buildinfo_metadata "$source_buildinfo_artifact" "$runtime_version"
  require_buildinfo_entry "$source_buildinfo_artifact" "$source_dsc"
else
  for source_artifact in \
    "$source_dsc" \
    "$source_orig_tar" \
    "$source_debian_tar"
  do
    require_changes_entry "$binary_changes_artifact" "$source_artifact"
  done

  require_buildinfo_entry "$binary_buildinfo_artifact" "$source_dsc"
fi

extract_root="$(mktemp -d)"
trap 'rm -rf "$extract_root"' EXIT

runtime_root="$extract_root/runtime"
dev_root="$extract_root/dev"
tools_root="$extract_root/tools"
udeb_root="$extract_root/udeb"
source_root="$extract_root/source"

dpkg-deb -x "$runtime_deb" "$runtime_root"
dpkg-deb -x "$dev_deb" "$dev_root"
dpkg-deb -x "$tools_deb" "$tools_root"
dpkg-deb -x "$udeb_artifact" "$udeb_root"
dpkg-source -x "$source_dsc" "$source_root" >/dev/null 2>&1

if [[ ! -f "$source_orig_tar" || ! -f "$source_debian_tar" ]]; then
  printf 'source package artifacts are incomplete for %s\n' "$runtime_version" >&2
  exit 1
fi

if grep -Fq 'profile=!noudeb' "$source_dsc"; then
  printf 'source package metadata %s still declares profile=!noudeb for libpng16-16-udeb\n' "$source_dsc" >&2
  exit 1
fi

validate_source_package_matches_tree "$source_root"
validate_safe_snapshot_tar_matches_tree "$safe_source_snapshot_tar"

multiarch_dir="$(find "$runtime_root/usr/lib" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | head -n1)"
if [[ -z "$multiarch_dir" ]]; then
  printf 'unable to determine multiarch runtime directory from %s\n' "$runtime_deb" >&2
  exit 1
fi

require_path "$runtime_root" "usr/lib/$multiarch_dir/libpng16.so.16.43.0"
require_symlink_target "$runtime_root" "usr/lib/$multiarch_dir/libpng16.so.16" "libpng16.so.16.43.0"

runtime_lib="$runtime_root/usr/lib/$multiarch_dir/libpng16.so.16.43.0"
for symbol_name in \
  png_destroy_read_struct \
  png_set_expand \
  png_set_gray_to_rgb \
  png_set_interlace_handling \
  png_set_palette_to_rgb \
  png_set_strip_16 \
  png_set_tRNS_to_alpha
do
  if ! objdump -T "$runtime_lib" | grep -Eq "[[:space:]]PNG16_0[[:space:]]+$symbol_name$"; then
    printf 'runtime package is missing PNG16_0 versioning for %s\n' "$symbol_name" >&2
    exit 1
  fi
done

for rel in \
  "usr/include/libpng16/png.h" \
  "usr/include/libpng16/pngconf.h" \
  "usr/include/libpng16/pnglibconf.h" \
  "usr/include/png.h" \
  "usr/include/pngconf.h" \
  "usr/include/pnglibconf.h" \
  "usr/include/libpng" \
  "usr/lib/$multiarch_dir/libpng16.a" \
  "usr/lib/$multiarch_dir/libpng.a" \
  "usr/lib/$multiarch_dir/libpng16.so" \
  "usr/lib/$multiarch_dir/libpng.so" \
  "usr/lib/$multiarch_dir/pkgconfig/libpng16.pc" \
  "usr/lib/$multiarch_dir/pkgconfig/libpng.pc" \
  "usr/bin/libpng16-config" \
  "usr/bin/libpng-config" \
  "usr/share/man/man1/libpng16-config.1.gz" \
  "usr/share/man/man1/libpng-config.1.gz" \
  "usr/share/man/man3/libpng.3.gz" \
  "usr/share/man/man5/png.5.gz"
do
  require_path "$dev_root" "$rel"
done

require_symlink_target "$dev_root" "usr/include/libpng" "libpng16"
require_symlink_target "$dev_root" "usr/include/png.h" "libpng16/png.h"
require_symlink_target "$dev_root" "usr/include/pngconf.h" "libpng16/pngconf.h"
require_symlink_target "$dev_root" "usr/include/pnglibconf.h" "libpng16/pnglibconf.h"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/libpng.a" "libpng16.a"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/libpng.so" "libpng16.so"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/pkgconfig/libpng.pc" "libpng16.pc"
require_symlink_target "$dev_root" "usr/bin/libpng-config" "libpng16-config"

examples_dir="$dev_root/usr/share/doc/libpng-dev/examples"
upstream_examples_root="$safe_dir"
for example_name in example.c pngtest.c pngtest.png; do
  require_path "$dev_root" "usr/share/doc/libpng-dev/examples/$example_name"
done

for example_name in example.c pngtest.c; do
  if ! cmp -s "$examples_dir/$example_name" "$upstream_examples_root/$example_name"; then
    printf 'packaged libpng-dev example payload diverged: %s\n' "$example_name" >&2
    exit 1
  fi
done

pngtest_description="$(file -b "$examples_dir/pngtest.png")"
if [[ "$pngtest_description" != *"PNG image data, 91 x 69"* ]]; then
  printf 'unexpected packaged pngtest.png description: %s\n' "$pngtest_description" >&2
  exit 1
fi

for rel in \
  "usr/bin/pngfix" \
  "usr/bin/png-fix-itxt"
do
  require_path "$tools_root" "$rel"
done

udeb_lib="$(find "$udeb_root/lib" -name 'libpng16.so.16.43.0' -print -quit)"
if [[ -z "$udeb_lib" ]]; then
  printf 'udeb payload is missing libpng16.so.16.43.0\n' >&2
  exit 1
fi

udeb_lib_dir="$(dirname "$udeb_lib")"
if [[ "$(readlink "$udeb_lib_dir/libpng16.so.16")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected udeb soname symlink target\n' >&2
  exit 1
fi

printf 'package artifacts passed for %s\n' "$runtime_version"
printf 'source package artifacts match the current safe packaging tree\n'
printf 'safe source snapshot tar matches the current tracked safe tree\n'
printf 'libpng-dev examples preserved under /usr/share/doc/libpng-dev/examples/\n'
