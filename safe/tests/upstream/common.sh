#!/usr/bin/env bash

if [[ -n "${LIBPNG_SAFE_UPSTREAM_COMMON_LOADED:-}" ]]; then
  return 0 2>/dev/null || exit 0
fi

readonly LIBPNG_SAFE_UPSTREAM_COMMON_LOADED=1

readonly upstream_script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly safe_dir="$(cd -- "$upstream_script_dir/../.." && pwd)"
readonly repo_root="$(cd -- "$safe_dir/.." && pwd)"
readonly original_root="$repo_root/original"
readonly upstream_root="$safe_dir"
for required_path in \
  "$upstream_root/pngtest.c" \
  "$upstream_root/pngtest.png" \
  "$upstream_root/contrib/tools/pngfix.c" \
  "$upstream_root/contrib/tools/png-fix-itxt.c" \
  "$upstream_root/contrib/tools/pngcp.c" \
  "$upstream_root/contrib/libtests/pngvalid.c" \
  "$upstream_root/contrib/libtests/pngunknown.c" \
  "$upstream_root/contrib/libtests/pngstest.c" \
  "$upstream_root/contrib/libtests/pngimage.c" \
  "$upstream_root/contrib/libtests/readpng.c" \
  "$upstream_root/contrib/libtests/tarith.c" \
  "$upstream_root/contrib/libtests/timepng.c" \
  "$upstream_root/contrib/pngsuite/basn0g08.png" \
  "$upstream_root/contrib/visupng/cexcept.h" \
  "$upstream_root/contrib/testpngs/badpal/regression-palette-8.png" \
  "$original_root/tests/pngtest-all" \
  "$original_root/tests/pngvalid-standard" \
  "$original_root/tests/pngunknown-discard" \
  "$original_root/tests/pngstest" \
  "$original_root/tests/pngstest-none" \
  "$original_root/tests/pngimage-quick" \
  "$original_root/tests/tarith-ascii" \
  "$original_root/pngtest.png" \
  "$original_root/contrib/pngsuite/basn0g08.png" \
  "$original_root/contrib/testpngs/badpal/regression-palette-8.png"; do
  if [[ ! -f "$required_path" ]]; then
    printf 'missing canonical in-tree upstream packaging input: %s\n' "$required_path" >&2
    exit 1
  fi
done
readonly profile="${PROFILE:-release}"
readonly target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
readonly profile_dir="$target_root/$profile"
readonly stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"

libpng_stage_shared_lib=""
libpng_stage_static_lib=""
libpng_stage_lib_dir=""
libpng_stage_include_dir=""
libpng_stage_header_dir=""

original_stage_workspace=""
original_stage_root=""
original_stage_shared_lib=""
original_stage_static_lib=""
original_stage_lib_dir=""
original_stage_include_dir=""
original_stage_header_dir=""
original_stage_pkgconfig_dir=""
original_stage_config_script=""

build_jobs() {
  if command -v nproc >/dev/null 2>&1; then
    nproc
    return 0
  fi

  getconf _NPROCESSORS_ONLN 2>/dev/null || printf '1\n'
}

detect_multiarch() {
  local value

  if [[ -n "${LIBPNG_MULTIARCH:-}" ]]; then
    printf '%s\n' "$LIBPNG_MULTIARCH"
    return 0
  fi

  if command -v dpkg-architecture >/dev/null 2>&1; then
    value="$(dpkg-architecture -qDEB_HOST_MULTIARCH 2>/dev/null || true)"
    if [[ -n "$value" ]]; then
      printf '%s\n' "$value"
      return 0
    fi
  fi

  if command -v gcc >/dev/null 2>&1; then
    value="$(gcc -print-multiarch 2>/dev/null || true)"
    if [[ -n "$value" ]]; then
      printf '%s\n' "$value"
      return 0
    fi
  fi

  case "$(uname -m)" in
    x86_64)
      printf 'x86_64-linux-gnu\n'
      ;;
    aarch64)
      printf 'aarch64-linux-gnu\n'
      ;;
    *)
      uname -m
      ;;
  esac
}

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
  libpng_stage_static_lib="$libpng_stage_lib_dir/libpng16.a"
  libpng_stage_include_dir="$stage_root/usr/include"
  libpng_stage_header_dir="$libpng_stage_include_dir/libpng16"

  if [[ ! -f "$libpng_stage_static_lib" ]]; then
    printf 'unable to locate staged libpng static library under %s\n' "$libpng_stage_lib_dir" >&2
    exit 1
  fi
}

ensure_safe_stage() {
  local current_shared="$profile_dir/libpng16.so"
  local current_static="$profile_dir/libpng16.a"
  local staged_shared=""
  local staged_static=""
  local staged_header_dir="$stage_root/usr/include/libpng16"

  if [[ -n "$libpng_stage_shared_lib" && -e "$libpng_stage_shared_lib" ]]; then
    staged_shared="$libpng_stage_shared_lib"
    staged_static="$libpng_stage_lib_dir/libpng16.a"
  elif [[ -d "$stage_root/usr" ]]; then
    staged_shared="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
    if [[ -n "$staged_shared" ]]; then
      staged_static="$(dirname "$staged_shared")/libpng16.a"
    fi
  fi

  if [[ -n "$staged_shared" && -e "$staged_shared" && -e "$staged_static" && -e "$current_shared" && -e "$current_static" ]] \
    && [[ ! "$current_shared" -nt "$staged_shared" ]] \
    && [[ ! "$current_static" -nt "$staged_static" ]] \
    && [[ ! "$safe_dir/include/png.h" -nt "$staged_header_dir/png.h" ]] \
    && [[ ! "$safe_dir/include/pngconf.h" -nt "$staged_header_dir/pngconf.h" ]] \
    && [[ ! "$safe_dir/include/pnglibconf.h" -nt "$staged_header_dir/pnglibconf.h" ]]; then
    locate_safe_stage
    return 0
  fi

  build_safe_stage
}

ensure_original_stage() {
  if [[ -n "$original_stage_header_dir" && -d "$original_stage_header_dir" ]]; then
    return 0
  fi

  original_stage_workspace="$(mktemp -d)"
  original_stage_root="$original_stage_workspace/install"
  original_stage_include_dir="$original_stage_root/usr/include"
  original_stage_header_dir="$original_stage_include_dir/libpng16"
  mkdir -p "$original_stage_header_dir"

  install -m 0644 "$repo_root/original/png.h" "$original_stage_header_dir/png.h"
  install -m 0644 "$repo_root/original/pngconf.h" "$original_stage_header_dir/pngconf.h"
  install -m 0644 "$safe_dir/include/pnglibconf.h" "$original_stage_header_dir/pnglibconf.h"
}

cleanup_original_stage() {
  if [[ -n "$original_stage_workspace" && -d "$original_stage_workspace" ]]; then
    rm -rf "$original_stage_workspace"
  fi

  original_stage_workspace=""
  original_stage_root=""
  original_stage_shared_lib=""
  original_stage_static_lib=""
  original_stage_lib_dir=""
  original_stage_include_dir=""
  original_stage_header_dir=""
  original_stage_pkgconfig_dir=""
  original_stage_config_script=""
}

extract_upstream_tests() {
  printf '%s\n' \
    pngtest-all \
    pngvalid-standard \
    pngunknown-discard \
    pngstest-none \
    pngimage-quick \
    tarith-ascii
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
    -I"$libpng_stage_header_dir"
    -I"$upstream_root/contrib/visupng"
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
    "$upstream_root/pngtest.c" \
    > "$dest"

  printf '%s\n' "$dest"
}

build_preserved_original_object() {
  local output="$1"
  local source="$2"
  local build_dir="$3"
  shift 3

  ensure_original_stage

  local -a cc_args=(
    -std=c99
    -Wall
    -Wextra
    -Werror
    -Wno-deprecated-declarations
    -DPNG_FREESTANDING_TESTS
    -I"$original_stage_header_dir"
  )
  cc_args+=("$@")
  cc_args+=(
    -c "$source"
    -o "$build_dir/$output.o"
  )

  cc "${cc_args[@]}"
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
        -I"$libpng_stage_header_dir" \
        "$pngtest_source" \
        -L"$libpng_stage_lib_dir" \
        -Wl,-rpath,"$libpng_stage_lib_dir" \
        -lpng16 -lz -lm \
        -o "$build_dir/pngtest"
      ;;
    pngvalid)
      compile_libpng_client pngvalid "$upstream_root/contrib/libtests/pngvalid.c" "$build_dir"
      ;;
    pngstest)
      compile_libpng_client pngstest "$upstream_root/contrib/libtests/pngstest.c" "$build_dir"
      ;;
    pngunknown)
      compile_libpng_client pngunknown "$upstream_root/contrib/libtests/pngunknown.c" "$build_dir"
      ;;
    pngimage)
      compile_libpng_client pngimage "$upstream_root/contrib/libtests/pngimage.c" "$build_dir"
      ;;
    tarith)
      compile_libpng_client tarith "$upstream_root/contrib/libtests/tarith.c" "$build_dir"
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
  local wrapper_path="$original_root/tests/$wrapper_name"

  if [[ ! -f "$wrapper_path" ]]; then
    printf 'missing upstream wrapper: %s\n' "$wrapper_path" >&2
    exit 1
  fi

  (
    cd "$build_dir"
    srcdir="$original_root"
    export srcdir
    exec /bin/sh "$wrapper_path"
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
  compile_libpng_client pngcp "$upstream_root/contrib/tools/pngcp.c" "$1"
}

build_pngfix_consumer() {
  compile_libpng_client pngfix "$upstream_root/contrib/tools/pngfix.c" "$1"
}

build_timepng_consumer() {
  compile_libpng_client timepng "$upstream_root/contrib/libtests/timepng.c" "$1"
}

build_png_fix_itxt_tool() {
  compile_standalone_tool png-fix-itxt "$upstream_root/contrib/tools/png-fix-itxt.c" "$1"
}

smoke_pngcp() {
  local build_dir="$1"
  local output="$build_dir/pngcp-fixed.png"

  "$build_dir/pngcp" \
    --fix-palette-index \
    "$upstream_root/contrib/testpngs/badpal/regression-palette-8.png" \
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
    "$upstream_root/pngtest.png"

  if [[ ! -s "$output" ]]; then
    printf 'pngfix did not produce an output file\n' >&2
    exit 1
  fi
}

smoke_timepng() {
  local build_dir="$1"
  "$build_dir/timepng" "$upstream_root/pngtest.png" >/dev/null
}

smoke_png_fix_itxt() {
  local build_dir="$1"
  local output="$build_dir/png-fix-itxt-output.png"

  "$build_dir/png-fix-itxt" \
    < "$upstream_root/pngtest.png" \
    > "$output"

  cmp -s "$upstream_root/pngtest.png" "$output"
}
