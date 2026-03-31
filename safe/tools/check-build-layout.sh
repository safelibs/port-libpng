#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"
layout_baseline="$safe_dir/abi/install-layout.txt"

build_args=(build --manifest-path "$safe_dir/Cargo.toml")
if [[ "$profile" == "release" ]]; then
  build_args+=(--release)
else
  build_args+=(--profile "$profile")
fi

baseline_has_path() {
  grep -qxF "$1" "$layout_baseline"
}

libdir_rel="$(grep -E '^usr/lib/.*/libpng16\.so\.16\.43\.0$' "$layout_baseline" | sed 's#/libpng16\.so\.16\.43\.0$##')"
if [[ -z "$libdir_rel" ]]; then
  printf 'unable to determine the frozen libdir from %s\n' "$layout_baseline" >&2
  exit 1
fi

package_only_paths=(
  "usr/bin/png-fix-itxt"
  "usr/bin/pngfix"
  "$libdir_rel/libpng.la"
  "$libdir_rel/libpng16.la"
  "usr/share/man/man3/libpng.3"
  "usr/share/man/man3/libpngpf.3"
  "usr/share/man/man5/png.5"
)

for rel in "${package_only_paths[@]}"; do
  if ! baseline_has_path "$rel"; then
    printf 'expected original-only install path missing from %s: %s\n' "$layout_baseline" "$rel" >&2
    exit 1
  fi
done

if baseline_has_path 'usr/include/libpng'; then
  printf 'original install-layout baseline unexpectedly already contains usr/include/libpng\n' >&2
  exit 1
fi

cargo "${build_args[@]}"
"$safe_dir/tools/stage-install-tree.sh"

actual_paths="$(mktemp)"
expected_paths="$(mktemp)"
trap 'rm -f "$actual_paths" "$expected_paths"' EXIT

find "$stage_root" -mindepth 1 \( -type f -o -type l \) -printf '%P\n' \
  | LC_ALL=C sort \
  > "$actual_paths"

grep -Fvx \
  -e 'usr/bin/png-fix-itxt' \
  -e 'usr/bin/pngfix' \
  -e "$libdir_rel/libpng.la" \
  -e "$libdir_rel/libpng16.la" \
  -e 'usr/share/man/man3/libpng.3' \
  -e 'usr/share/man/man3/libpngpf.3' \
  -e 'usr/share/man/man5/png.5' \
  "$layout_baseline" \
  > "$expected_paths"
printf '%s\n' 'usr/include/libpng' >> "$expected_paths"
LC_ALL=C sort -u -o "$expected_paths" "$expected_paths"

if ! diff -u "$expected_paths" "$actual_paths"; then
  printf 'staged build layout diverged from the expected safe delta over %s\n' "$layout_baseline" >&2
  exit 1
fi

libdir="$stage_root/$libdir_rel"

for staged_file in \
  "$stage_root/usr/bin/libpng16-config" \
  "$libdir/libpng16.so.16.43.0" \
  "$libdir/libpng16.a" \
  "$libdir/pkgconfig/libpng16.pc"
do
  if [[ ! -f "$staged_file" || -L "$staged_file" ]]; then
    printf 'staged install artifact must be a regular file: %s\n' "$staged_file" >&2
    exit 1
  fi
done

if [[ ! -x "$stage_root/usr/bin/libpng16-config" ]]; then
  printf 'staged config script is not executable: %s\n' "$stage_root/usr/bin/libpng16-config" >&2
  exit 1
fi

for staged_file in "$libdir/libpng16.so.16.43.0" "$libdir/libpng16.a"; do
  if [[ "$(readlink -f "$staged_file")" != "$staged_file" ]]; then
    printf 'staged install artifact resolves outside the staged tree: %s\n' "$staged_file" >&2
    exit 1
  fi
done

if [[ "$(readlink "$libdir/libpng16.so.16")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so.16 link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng16.so")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng.so")" != "libpng16.so" ]]; then
  printf 'unexpected libpng.so link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng.a")" != "libpng16.a" ]]; then
  printf 'unexpected libpng.a link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/pkgconfig/libpng.pc")" != "libpng16.pc" ]]; then
  printf 'unexpected libpng.pc link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$stage_root/usr/bin/libpng-config")" != "libpng16-config" ]]; then
  printf 'unexpected libpng-config link target\n' >&2
  exit 1
fi

for header in png.h pngconf.h pnglibconf.h; do
  if [[ "$(readlink "$stage_root/usr/include/$header")" != "libpng16/$header" ]]; then
    printf 'unexpected top-level header link target for %s\n' "$header" >&2
    exit 1
  fi
done

if [[ "$(readlink "$stage_root/usr/include/libpng")" != "libpng16" ]]; then
  printf 'unexpected include compatibility symlink for %s\n' "$stage_root/usr/include/libpng" >&2
  exit 1
fi

printf 'staged build layout matches the expected safe install-surface delta from the frozen original baseline\n'
