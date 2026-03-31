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
"$safe_dir/tools/stage-install-tree.sh"

lib_path="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
if [[ -z "$lib_path" ]]; then
  printf 'unable to locate staged libpng shared library under %s\n' "$stage_root/usr/lib" >&2
  exit 1
fi

lib_dir="$(dirname "$lib_path")"
build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
  "$repo_root/original/contrib/libtests/pngunknown.c" \
  -L"$lib_dir" \
  -Wl,-rpath,"$lib_dir" \
  -lpng16 -lz -lm \
  -o "$build_dir/pngunknown"

cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
  "$repo_root/original/contrib/libtests/readpng.c" \
  -DPNG_FREESTANDING_TESTS \
  -I"$stage_root/usr/include" \
  -L"$lib_dir" \
  -Wl,-rpath,"$lib_dir" \
  -lpng16 -lz -lm \
  -o "$build_dir/readpng"

pushd "$build_dir" >/dev/null
for wrapper in \
  "$repo_root/original/tests/pngunknown-discard" \
  "$repo_root/original/tests/pngunknown-save" \
  "$repo_root/original/tests/pngunknown-if-safe" \
  "$repo_root/original/tests/pngunknown-vpAg" \
  "$repo_root/original/tests/pngunknown-sTER" \
  "$repo_root/original/tests/pngunknown-IDAT" \
  "$repo_root/original/tests/pngunknown-sAPI"; do
  srcdir="$repo_root/original" "$wrapper"
done
popd >/dev/null

"$build_dir/readpng" < "$repo_root/original/pngtest.png" >/dev/null

printf 'upstream pngunknown wrappers and readpng passed against the staged safe libpng build\n'
