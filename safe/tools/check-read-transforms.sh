#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
tests_dir="$safe_dir/tests/read-transforms"
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
done

"$build_dir/update_info_driver" \
  "$repo_root/original/contrib/testpngs/palette-8-sRGB-tRNS.png" \
  "$repo_root/original/contrib/testpngs/gray-2-sRGB-tRNS.png" \
  "$repo_root/original/contrib/testpngs/rgb-16-1.8.png" \
  "$repo_root/original/contrib/testpngs/gray-16.png" \
  "$repo_root/original/contrib/testpngs/rgb-alpha-16-linear.png" \
  "$repo_root/original/contrib/pngsuite/interlaced/ibasn0g01.png"

"$build_dir/read_png_driver" \
  "$repo_root/original/contrib/testpngs/palette-4-tRNS.png" \
  "$repo_root/original/contrib/testpngs/badpal/regression-palette-8.png" \
  "$repo_root/original/contrib/testpngs/crashers/bad_iCCP.png"

"$build_dir/colorspace_driver"

"$build_dir/simplified_read_driver" \
  "$repo_root/original/contrib/testpngs/rgb-8.png" \
  "$repo_root/original/contrib/pngsuite/ibasn6a16.png" \
  "$repo_root/original/contrib/testpngs/gray-2-sRGB-tRNS.png" \
  "$repo_root/original/contrib/testpngs/palette-1-sRGB-tRNS.png" \
  "$repo_root/original/contrib/testpngs/rgb-16-1.8.png" \
  "$repo_root/original/contrib/testpngs/gray-2-1.8.png"

printf 'read-transform and simplified-read smoke drivers passed against the staged safe libpng build\n'
