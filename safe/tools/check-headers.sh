#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
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

for header in png.h pngconf.h pnglibconf.h; do
  baseline="$safe_dir/include/$header"
  staged="$stage_root/usr/include/libpng16/$header"

  if [[ ! -f "$staged" ]]; then
    printf 'missing staged header: %s\n' "$staged" >&2
    exit 1
  fi

  if ! cmp -s "$baseline" "$staged"; then
    printf 'header mismatch: %s\n' "$header" >&2
    exit 1
  fi

  top_level="$stage_root/usr/include/$header"
  if [[ "$(readlink "$top_level")" != "libpng16/$header" ]]; then
    printf 'unexpected top-level header link target for %s\n' "$top_level" >&2
    exit 1
  fi

  if ! cmp -s "$baseline" "$top_level"; then
    printf 'top-level header content mismatch: %s\n' "$top_level" >&2
    exit 1
  fi
done

header="$safe_dir/include/png.h"
if ! rg -q 'PNG_(?:EXPORT|EXPORTA|FP_EXPORT|FIXED_EXPORT)\(249,' "$header"; then
  printf 'png.h no longer exposes ordinal 249\n' >&2
  exit 1
fi

if rg -q 'PNG_(?:EXPORT|EXPORTA|FP_EXPORT|FIXED_EXPORT)\(200,' "$header"; then
  printf 'png.h unexpectedly reintroduced exported ordinal 200\n' >&2
  exit 1
fi

if ! rg -q 'PNG_REMOVED\(200,' "$header"; then
  printf 'png.h no longer marks ordinal 200 as intentionally unused\n' >&2
  exit 1
fi

if rg -q 'PNG_(?:EXPORT|EXPORTA|FP_EXPORT|FIXED_EXPORT|REMOVED)\(2(5[0-9]|6[0-9]),' "$header"; then
  printf 'png.h unexpectedly contains APNG ordinals 250-269\n' >&2
  exit 1
fi

printf 'header baseline matches staged safe headers\n'
