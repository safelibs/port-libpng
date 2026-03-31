#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
tests_dir="$safe_dir/tests/core-smoke"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"

build_args=(build --manifest-path "$safe_dir/Cargo.toml")
if [[ "$profile" == "release" ]]; then
  build_args+=(--release)
else
  build_args+=(--profile "$profile")
fi

cargo "${build_args[@]}"
"$safe_dir/tools/stage-install-tree.sh"

lib_path="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
if [[ -z "$lib_path" ]]; then
  printf 'unable to locate staged libpng shared library under %s\n' "$stage_root/usr/lib" >&2
  exit 1
fi

lib_dir="$(dirname "$lib_path")"
include_dir="$stage_root/usr/include"
build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

for src in "$tests_dir"/*.c; do
  exe="$build_dir/$(basename "${src%.c}")"
  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    -I"$include_dir" \
    "$src" \
    -L"$lib_dir" \
    -Wl,-rpath,"$lib_dir" \
    -lpng16 -lz -lm \
    -o "$exe"
  "$exe"
done

printf 'core smoke drivers passed against the staged safe libpng build\n'
