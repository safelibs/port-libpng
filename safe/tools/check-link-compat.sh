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

shared_dir="$build_dir/shared"
static_dir="$build_dir/static"
mkdir -p "$shared_dir" "$static_dir"

compile_object() {
  local output="$1"
  local source="$2"

  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    -I"$repo_root/original" \
    -I"$repo_root/original/contrib/visupng" \
    -c "$source" \
    -o "$build_dir/$output.o"
}

link_shared_program() {
  local name="$1"

  cc "$build_dir/$name.o" \
    -L"$lib_dir" \
    -Wl,-rpath,"$lib_dir" \
    -lpng16 -lz -lm \
    -o "$shared_dir/$name"
}

link_static_program() {
  local name="$1"

  cc "$build_dir/$name.o" \
    "$static_lib" -lz -lm \
    -o "$static_dir/$name"
}

run_wrapper() {
  local mode_dir="$1"
  local wrapper_name="$2"
  local wrapper="$repo_root/original/tests/$wrapper_name"

  if [[ ! -f "$wrapper" ]]; then
    printf 'missing upstream wrapper: %s\n' "$wrapper" >&2
    exit 1
  fi

  pushd "$mode_dir" >/dev/null
  srcdir="$repo_root/original" sh "$wrapper"
  popd >/dev/null
}

run_pngcp() {
  local mode_dir="$1"
  local output="$mode_dir/pngcp-fixed.png"

  "$mode_dir/pngcp" \
    --fix-palette-index \
    "$repo_root/original/contrib/testpngs/badpal/regression-palette-8.png" \
    "$output"

  if [[ ! -s "$output" ]]; then
    printf 'pngcp did not produce an output file in %s\n' "$mode_dir" >&2
    exit 1
  fi
}

run_timepng() {
  local mode_dir="$1"
  "$mode_dir/timepng" "$repo_root/original/pngtest.png" >/dev/null
}

run_pngtest() {
  local mode_dir="$1"
  "$mode_dir/pngtest" --strict "$repo_root/original/pngtest.png" >/dev/null
}

run_mode_matrix() {
  local mode_label="$1"
  local mode_dir="$2"

  run_pngtest "$mode_dir"
  run_wrapper "$mode_dir" pngunknown-discard
  run_wrapper "$mode_dir" pngstest-none
  run_wrapper "$mode_dir" pngvalid-standard
  run_wrapper "$mode_dir" pngimage-quick
  run_wrapper "$mode_dir" tarith-ascii
  run_pngcp "$mode_dir"
  run_timepng "$mode_dir"

  printf '%s link-compatibility matrix passed for pngtest, pngunknown, pngstest, pngvalid, pngimage, tarith, pngcp, and timepng\n' "$mode_label"
}

compile_object pngtest "$repo_root/original/pngtest.c"
compile_object pngunknown "$repo_root/original/contrib/libtests/pngunknown.c"
compile_object pngstest "$repo_root/original/contrib/libtests/pngstest.c"
compile_object pngvalid "$repo_root/original/contrib/libtests/pngvalid.c"
compile_object pngimage "$repo_root/original/contrib/libtests/pngimage.c"
compile_object tarith "$repo_root/original/contrib/libtests/tarith.c"
compile_object pngcp "$repo_root/original/contrib/tools/pngcp.c"
compile_object timepng "$repo_root/original/contrib/libtests/timepng.c"

for program in pngtest pngunknown pngstest pngvalid pngimage tarith pngcp timepng; do
  link_shared_program "$program"
  link_static_program "$program"
done

run_mode_matrix "shared" "$shared_dir"
run_mode_matrix "static" "$static_dir"

cc "$build_dir/pngtest.o" \
  "$static_lib" -lz -lm \
  -o "$build_dir/pngtest-static"

pushd "$repo_root/original" >/dev/null
"$build_dir/pngtest-static" >/dev/null
popd >/dev/null

printf 'debian pngtest-static scenario passed with original object reuse against the staged safe static archive\n'
