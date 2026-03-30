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

shared_lib="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
static_lib="$(find "$stage_root/usr/lib" -name 'libpng16.a' -print -quit)"
if [[ -z "$shared_lib" || -z "$static_lib" ]]; then
  printf 'unable to locate staged safe libpng shared/static libraries under %s\n' "$stage_root/usr/lib" >&2
  exit 1
fi

lib_dir="$(dirname "$shared_lib")"
build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

obj="$build_dir/pngunknown.o"
cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
  -c "$repo_root/original/contrib/libtests/pngunknown.c" \
  -o "$obj"

shared_exe="$build_dir/pngunknown-shared"
static_exe="$build_dir/pngunknown-static"

cc "$obj" \
  -L"$lib_dir" \
  -Wl,-rpath,"$lib_dir" \
  -lpng16 -lz -lm \
  -o "$shared_exe"

cc "$obj" \
  "$static_lib" -lz -lm \
  -o "$static_exe"

"$shared_exe" --strict default=discard "$repo_root/original/pngtest.png"
"$static_exe" --strict default=discard "$repo_root/original/pngtest.png"

printf 'original-header objects linked and ran against staged shared and static safe libpng builds\n'
