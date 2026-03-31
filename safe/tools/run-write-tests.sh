#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"

mapfile -t available_wrappers < <(
  find "$repo_root/original/tests" -maxdepth 1 -type f -name 'pngstest-*' -printf '%f\n' | sort
)

if [[ $# -eq 0 ]]; then
  printf 'usage: %s <pngstest-wrapper> [<pngstest-wrapper> ...]\n' "${0##*/}" >&2
  printf 'available wrappers:' >&2
  for wrapper in "${available_wrappers[@]}"; do
    printf ' %s' "$wrapper" >&2
  done
  printf '\n' >&2
  exit 1
fi

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

compile_program() {
  local output="$1"
  local source="$2"

  cc -std=c99 -Wall -Wextra -Werror -Wno-deprecated-declarations \
    -DPNG_FREESTANDING_TESTS \
    -I"$include_dir" \
    -I"$repo_root/original" \
    -I"$repo_root/original/contrib/visupng" \
    "$source" \
    -L"$lib_dir" \
    -Wl,-rpath,"$lib_dir" \
    -lpng16 -lz -lm \
    -o "$build_dir/$output"
}

compile_program pngstest "$repo_root/original/contrib/libtests/pngstest.c"

pushd "$build_dir" >/dev/null
for wrapper_name in "$@"; do
  if [[ "$wrapper_name" == "pngstest" ]]; then
    printf 'original/tests/pngstest is a helper script, not a standalone verifier\n' >&2
    exit 1
  fi

  if [[ ! "$wrapper_name" =~ ^pngstest- ]]; then
    printf 'unsupported write wrapper: %s\n' "$wrapper_name" >&2
    exit 1
  fi

  wrapper="$repo_root/original/tests/$wrapper_name"
  if [[ ! -f "$wrapper" ]]; then
    printf 'missing upstream wrapper: %s\n' "$wrapper" >&2
    exit 1
  fi

  srcdir="$repo_root/original" sh "$wrapper"
done
popd >/dev/null

printf 'upstream pngstest wrapper matrix passed against the staged safe libpng build\n'
