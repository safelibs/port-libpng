#!/usr/bin/env bash

if [[ -n "${LIBPNG_SAFE_UPSTREAM_COMMON_LOADED:-}" ]]; then
  return 0 2>/dev/null || exit 0
fi

readonly LIBPNG_SAFE_UPSTREAM_COMMON_LOADED=1

readonly upstream_script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly safe_dir="$(cd -- "$upstream_script_dir/../.." && pwd)"
readonly repo_root="$(cd -- "$safe_dir/.." && pwd)"
readonly profile="${PROFILE:-release}"
readonly target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
readonly stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"

libpng_stage_shared_lib=""
libpng_stage_lib_dir=""
libpng_stage_include_dir=""

build_safe_stage() {
  local build_args=(build --manifest-path "$safe_dir/Cargo.toml")

  if [[ "$profile" == "release" ]]; then
    build_args+=(--release)
  else
    build_args+=(--profile "$profile")
  fi

  cargo "${build_args[@]}"
  "$safe_dir/tools/stage-install-tree.sh"
  locate_safe_stage
}

locate_safe_stage() {
  libpng_stage_shared_lib="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
  if [[ -z "$libpng_stage_shared_lib" ]]; then
    printf 'unable to locate staged libpng shared library under %s\n' "$stage_root/usr/lib" >&2
    exit 1
  fi

  libpng_stage_lib_dir="$(dirname "$libpng_stage_shared_lib")"
  libpng_stage_include_dir="$stage_root/usr/include"
}

ensure_safe_stage() {
  if [[ -n "$libpng_stage_shared_lib" && -e "$libpng_stage_shared_lib" ]]; then
    return 0
  fi

  if [[ -d "$stage_root/usr" ]]; then
    locate_safe_stage
  else
    build_safe_stage
  fi
}

extract_upstream_tests() {
  awk '
    !in_tests && /^TESTS =[[:space:]]*\\/ {
      in_tests = 1
      line = $0
      sub(/^TESTS =[[:space:]]*\\/, "", line)
      print line
      next
    }
    in_tests {
      if ($0 ~ /^endif$/) {
        exit
      }
      print $0
    }
  ' "$repo_root/original/Makefile.am" \
    | tr '\\' '\n' \
    | xargs -n1 \
    | sed '/^$/d; s#^tests/##'
}

wrapper_program_for() {
  case "$1" in
    pngtest-all)
      printf 'pngtest\n'
      ;;
    pngvalid-*)
      printf 'pngvalid\n'
      ;;
    pngstest-*)
      printf 'pngstest\n'
      ;;
    pngunknown-*)
      printf 'pngunknown\n'
      ;;
    pngimage-*)
      printf 'pngimage\n'
      ;;
    tarith-*)
      printf 'tarith\n'
      ;;
    *)
      printf 'unsupported upstream wrapper: %s\n' "$1" >&2
      exit 1
      ;;
  esac
}

compile_libpng_client() {
  local output="$1"
  local source="$2"
  local build_dir="$3"
  shift 3

  ensure_safe_stage

  local -a cc_args=(
    -std=c99
    -Wall
    -Wextra
    -Werror
    -Wno-deprecated-declarations
    -DPNG_FREESTANDING_TESTS
    -I"$libpng_stage_include_dir"
    -I"$repo_root/original"
    -I"$repo_root/original/contrib/visupng"
  )
  cc_args+=("$@")
  cc_args+=(
    "$source"
    -L"$libpng_stage_lib_dir"
    -Wl,-rpath,"$libpng_stage_lib_dir"
    -lpng16
    -lz
    -lm
    -o "$build_dir/$output"
  )

  cc "${cc_args[@]}"
}

compile_standalone_tool() {
  local output="$1"
  local source="$2"
  local build_dir="$3"

  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    "$source" \
    -lz \
    -o "$build_dir/$output"
}

prepare_pngtest_source() {
  local build_dir="$1"
  local dest="$build_dir/pngtest.c"

  sed 's/^#include "png.h"$/#include <png.h>/' \
    "$repo_root/original/pngtest.c" \
    > "$dest"

  printf '%s\n' "$dest"
}

compile_wrapper_program() {
  local program="$1"
  local build_dir="$2"
  local pngtest_source

  case "$program" in
    pngtest)
      ensure_safe_stage
      pngtest_source="$(prepare_pngtest_source "$build_dir")"
      cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
        -I"$libpng_stage_include_dir" \
        -I"$repo_root/original" \
        "$pngtest_source" \
        -L"$libpng_stage_lib_dir" \
        -Wl,-rpath,"$libpng_stage_lib_dir" \
        -lpng16 -lz -lm \
        -o "$build_dir/pngtest"
      ;;
    pngvalid)
      compile_libpng_client pngvalid "$repo_root/original/contrib/libtests/pngvalid.c" "$build_dir"
      ;;
    pngstest)
      compile_libpng_client pngstest "$repo_root/original/contrib/libtests/pngstest.c" "$build_dir"
      ;;
    pngunknown)
      compile_libpng_client pngunknown "$repo_root/original/contrib/libtests/pngunknown.c" "$build_dir"
      ;;
    pngimage)
      compile_libpng_client pngimage "$repo_root/original/contrib/libtests/pngimage.c" "$build_dir"
      ;;
    tarith)
      compile_libpng_client tarith "$repo_root/original/contrib/libtests/tarith.c" "$build_dir"
      ;;
    *)
      printf 'unsupported upstream wrapper program: %s\n' "$program" >&2
      exit 1
      ;;
  esac
}

run_original_wrapper() {
  local wrapper_name="$1"
  local build_dir="$2"
  local wrapper="$repo_root/original/tests/$wrapper_name"

  if [[ ! -f "$wrapper" ]]; then
    printf 'missing upstream wrapper: %s\n' "$wrapper" >&2
    exit 1
  fi

  (
    cd "$build_dir"
    srcdir="$repo_root/original" sh "$wrapper"
  )
}

run_wrapper_case() {
  local wrapper_name="$1"
  local build_dir="$2"
  local program

  program="$(wrapper_program_for "$wrapper_name")"
  compile_wrapper_program "$program" "$build_dir"
  run_original_wrapper "$wrapper_name" "$build_dir"
}

build_pngcp_consumer() {
  compile_libpng_client pngcp "$repo_root/original/contrib/tools/pngcp.c" "$1"
}

build_pngfix_consumer() {
  compile_libpng_client pngfix "$repo_root/original/contrib/tools/pngfix.c" "$1"
}

build_timepng_consumer() {
  compile_libpng_client timepng "$repo_root/original/contrib/libtests/timepng.c" "$1"
}

build_png_fix_itxt_tool() {
  compile_standalone_tool png-fix-itxt "$repo_root/original/contrib/tools/png-fix-itxt.c" "$1"
}

smoke_pngcp() {
  local build_dir="$1"
  local output="$build_dir/pngcp-fixed.png"

  "$build_dir/pngcp" \
    --fix-palette-index \
    "$repo_root/original/contrib/testpngs/badpal/regression-palette-8.png" \
    "$output"

  if [[ ! -s "$output" ]]; then
    printf 'pngcp did not produce an output file\n' >&2
    exit 1
  fi
}

smoke_pngfix() {
  local build_dir="$1"
  local output="$build_dir/pngfix-output.png"

  "$build_dir/pngfix" \
    "--out=$output" \
    "$repo_root/original/pngtest.png"

  if [[ ! -s "$output" ]]; then
    printf 'pngfix did not produce an output file\n' >&2
    exit 1
  fi
}

smoke_timepng() {
  local build_dir="$1"
  "$build_dir/timepng" "$repo_root/original/pngtest.png" >/dev/null
}

smoke_png_fix_itxt() {
  local build_dir="$1"
  local output="$build_dir/png-fix-itxt-output.png"

  "$build_dir/png-fix-itxt" \
    < "$repo_root/original/pngtest.png" \
    > "$output"

  cmp -s "$repo_root/original/pngtest.png" "$output"
}
