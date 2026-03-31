#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
source "$safe_dir/tests/upstream/common.sh"

ensure_safe_stage
ensure_original_stage

build_root="$(mktemp -d)"
trap 'rm -rf "$build_root"; cleanup_original_stage' EXIT

staged_header_objects="$build_root/staged-header-objects"
staged_header_shared="$build_root/staged-header-shared"
staged_header_static="$build_root/staged-header-static"
original_header_objects="$build_root/original-header-objects"
old_object_shared="$build_root/old-object-shared"
old_object_static="$build_root/old-object-static"

mkdir -p \
  "$staged_header_objects" \
  "$staged_header_shared" \
  "$staged_header_static" \
  "$original_header_objects" \
  "$old_object_shared" \
  "$old_object_static"

programs=(
  pngtest
  pngunknown
  pngstest
  pngimage
  pngcp
  timepng
)

compile_safe_header_object() {
  local output="$1"
  local source="$2"
  shift 2

  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    -DPNG_FREESTANDING_TESTS \
    -I"$libpng_stage_header_dir" \
    "$@" \
    -c "$source" \
    -o "$staged_header_objects/$output.o"
}

link_shared_program() {
  local object_dir="$1"
  local output_dir="$2"
  local name="$3"

  cc "$object_dir/$name.o" \
    -L"$libpng_stage_lib_dir" \
    -Wl,-rpath,"$libpng_stage_lib_dir" \
    -lpng16 -lz -lm \
    -o "$output_dir/$name"
}

link_static_program() {
  local object_dir="$1"
  local output_dir="$2"
  local name="$3"

  cc "$object_dir/$name.o" \
    "$libpng_stage_static_lib" -lz -lm \
    -o "$output_dir/$name"
}

run_pngtest() {
  local mode_dir="$1"
  "$mode_dir/pngtest" --strict "$repo_root/original/pngtest.png" >/dev/null
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

run_lane_matrix() {
  local lane_label="$1"
  local mode_dir="$2"

  run_pngtest "$mode_dir"
  run_original_wrapper pngunknown-discard "$mode_dir"
  run_original_wrapper pngstest-none "$mode_dir"
  run_original_wrapper pngimage-quick "$mode_dir"
  run_pngcp "$mode_dir"
  run_timepng "$mode_dir"

  printf '%s lane passed for pngtest, pngunknown, pngstest, pngimage, pngcp, and timepng\n' \
    "$lane_label"
}

staged_pngtest_source="$(prepare_pngtest_source "$staged_header_objects")"
compile_safe_header_object pngtest "$staged_pngtest_source"
compile_safe_header_object pngunknown "$repo_root/original/contrib/libtests/pngunknown.c"
compile_safe_header_object pngstest "$repo_root/original/contrib/libtests/pngstest.c"
compile_safe_header_object pngimage "$repo_root/original/contrib/libtests/pngimage.c"
compile_safe_header_object pngcp "$repo_root/original/contrib/tools/pngcp.c"
compile_safe_header_object timepng "$repo_root/original/contrib/libtests/timepng.c"

original_pngtest_source="$(prepare_pngtest_source "$original_header_objects")"
build_preserved_original_object pngtest "$original_pngtest_source" "$original_header_objects"
build_preserved_original_object pngunknown "$repo_root/original/contrib/libtests/pngunknown.c" "$original_header_objects"
build_preserved_original_object pngstest "$repo_root/original/contrib/libtests/pngstest.c" "$original_header_objects"
build_preserved_original_object pngimage "$repo_root/original/contrib/libtests/pngimage.c" "$original_header_objects"
build_preserved_original_object pngcp "$repo_root/original/contrib/tools/pngcp.c" "$original_header_objects"
build_preserved_original_object timepng "$repo_root/original/contrib/libtests/timepng.c" "$original_header_objects"

for program in "${programs[@]}"; do
  link_shared_program "$staged_header_objects" "$staged_header_shared" "$program"
  link_static_program "$staged_header_objects" "$staged_header_static" "$program"
  link_shared_program "$original_header_objects" "$old_object_shared" "$program"
  link_static_program "$original_header_objects" "$old_object_static" "$program"
done

run_lane_matrix "staged-header shared" "$staged_header_shared"
run_lane_matrix "staged-header static" "$staged_header_static"
run_lane_matrix "old-object shared" "$old_object_shared"
run_lane_matrix "old-object static" "$old_object_static"

printf 'true old-object relinks reused the original-built .o files against both staged safe libraries without recompiling them\n'
