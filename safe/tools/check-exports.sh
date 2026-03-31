#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"
baseline_exports="$safe_dir/abi/exports.txt"
layout_baseline="$safe_dir/abi/install-layout.txt"
version_script="$safe_dir/abi/libpng.vers"

build_args=(build --manifest-path "$safe_dir/Cargo.toml")
if [[ "$profile" == "release" ]]; then
  build_args+=(--release)
else
  build_args+=(--profile "$profile")
fi

cargo "${build_args[@]}"
"$safe_dir/tools/stage-install-tree.sh"

lib_rel="$(grep -E '^usr/lib/.*/libpng16\.so\.16\.43\.0$' "$layout_baseline")"
lib_path="$stage_root/$lib_rel"

if [[ ! -f "$lib_path" ]]; then
  printf 'missing staged shared library: %s\n' "$lib_path" >&2
  exit 1
fi

actual_exports="$(mktemp)"
version_script_exports="$(mktemp)"
versioned_exports="$(mktemp)"
trap 'rm -f "$actual_exports" "$version_script_exports" "$versioned_exports"' EXIT

readelf --dyn-syms --wide "$lib_path" \
  | awk '/GLOBAL/ && /DEFAULT/ {print $NF}' \
  | grep '^png_' \
  | sed 's/@.*$//' \
  | LC_ALL=C sort -u \
  > "$actual_exports"

awk '
  /^PNG16_0[[:space:]]*{global:/ {
    in_global = 1
    sub(/^.*{global:[[:space:]]*/, "", $0)
    if (length($0) != 0) {
      print $0
    }
    next
  }
  in_global {
    if ($0 ~ /^[[:space:]]*local:[[:space:]]*\*;[[:space:]]*};[[:space:]]*$/) {
      exit
    }
    print $0
  }
' "$version_script" \
  | tr ';' '\n' \
  | sed 's/^[[:space:]]*//;s/[[:space:]]*$//' \
  | grep '^png_' \
  | LC_ALL=C sort -u \
  > "$version_script_exports"

objdump -T "$lib_path" \
  | awk '$6 == "PNG16_0" {print $7}' \
  | grep '^png_' \
  | LC_ALL=C sort -u \
  > "$versioned_exports"

if ! diff -u "$baseline_exports" "$version_script_exports"; then
  printf 'Linux version script diverged from the frozen export baseline\n' >&2
  exit 1
fi

if ! diff -u "$baseline_exports" "$actual_exports"; then
  printf 'shared-library export set diverged from frozen baseline\n' >&2
  exit 1
fi

if ! diff -u "$baseline_exports" "$versioned_exports"; then
  printf 'staged shared library no longer versions every frozen export as PNG16_0\n' >&2
  exit 1
fi

count="$(wc -l < "$actual_exports" | tr -d ' ')"
if [[ "$count" != "246" ]]; then
  printf 'unexpected shared export count: expected 246, found %s\n' "$count" >&2
  exit 1
fi

if grep -qx 'png_err' "$actual_exports"; then
  printf 'png_err unexpectedly exported\n' >&2
  exit 1
fi

if grep -qx 'png_set_strip_error_numbers' "$actual_exports"; then
  printf 'png_set_strip_error_numbers unexpectedly exported\n' >&2
  exit 1
fi

if ! readelf --version-info "$lib_path" | grep -q 'Name: PNG16_0'; then
  printf 'version script PNG16_0 was not recorded in %s\n' "$lib_path" >&2
  exit 1
fi

printf 'export baseline and PNG16_0 versioning match the staged safe shared library\n'
