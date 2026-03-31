#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
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

lib_path="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
if [[ -z "$lib_path" ]]; then
  printf 'unable to locate staged libpng shared library under %s\n' "$stage_root/usr/lib" >&2
  exit 1
fi

lib_dir="$(dirname "$lib_path")"
include_dir="$stage_root/usr/include"
build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

compile_libpng_client() {
  local output="$1"
  local source="$2"

  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    -DPNG_FREESTANDING_TESTS \
    -I"$include_dir" \
    -I"$repo_root/original" \
    "$source" \
    -L"$lib_dir" \
    -Wl,-rpath,"$lib_dir" \
    -lpng16 -lz -lm \
    -o "$build_dir/$output"
}

compile_libpng_client pngcp "$repo_root/original/contrib/tools/pngcp.c"
compile_libpng_client pngfix "$repo_root/original/contrib/tools/pngfix.c"
compile_libpng_client timepng "$repo_root/original/contrib/libtests/timepng.c"

cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
  "$repo_root/original/contrib/tools/png-fix-itxt.c" \
  -lz \
  -o "$build_dir/png-fix-itxt"

pngcp_output="$build_dir/pngcp-fixed.png"
"$build_dir/pngcp" \
  --fix-palette-index \
  "$repo_root/original/contrib/testpngs/badpal/regression-palette-8.png" \
  "$pngcp_output"

if [[ ! -s "$pngcp_output" ]]; then
  printf 'pngcp did not produce an output file for the invalid-index path\n' >&2
  exit 1
fi

pngfix_output="$build_dir/pngfix-output.png"
"$build_dir/pngfix" \
  "--out=$pngfix_output" \
  "$repo_root/original/pngtest.png"

if [[ ! -s "$pngfix_output" ]]; then
  printf 'pngfix did not produce an output file\n' >&2
  exit 1
fi

"$build_dir/timepng" "$repo_root/original/pngtest.png" >/dev/null

png_fix_itxt_output="$build_dir/png-fix-itxt-output.png"
"$build_dir/png-fix-itxt" \
  < "$repo_root/original/pngtest.png" \
  > "$png_fix_itxt_output"

cmp -s "$repo_root/original/pngtest.png" "$png_fix_itxt_output"

printf 'libpng C consumer smokes passed for pngcp, pngfix, and timepng\n'
printf 'standalone png-fix-itxt smoke passed separately from libpng ABI coverage\n'
