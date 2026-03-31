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
"$safe_dir/tools/stage-install-tree.sh"

actual_headers="$(mktemp)"
expected_headers="$(mktemp)"
trap 'rm -f "$actual_headers" "$expected_headers"' EXIT

find "$stage_root/usr/include" -mindepth 1 \( -type f -o -type l \) -printf '%P\n' \
  | LC_ALL=C sort \
  > "$actual_headers"

cat <<'EOF' | LC_ALL=C sort > "$expected_headers"
libpng
libpng16/png.h
libpng16/pngconf.h
libpng16/pnglibconf.h
png.h
pngconf.h
pnglibconf.h
EOF

if ! diff -u "$expected_headers" "$actual_headers"; then
  printf 'staged header layout diverged from the exact current contract\n' >&2
  exit 1
fi

for header in png.h pngconf.h pnglibconf.h; do
  baseline="$safe_dir/include/$header"
  staged="$stage_root/usr/include/libpng16/$header"
  top_level="$stage_root/usr/include/$header"

  if [[ ! -f "$staged" || -L "$staged" ]]; then
    printf 'staged libpng16 header must be a regular file: %s\n' "$staged" >&2
    exit 1
  fi

  if ! cmp -s "$baseline" "$staged"; then
    printf 'header mismatch: %s\n' "$header" >&2
    exit 1
  fi

  if [[ "$(readlink "$top_level")" != "libpng16/$header" ]]; then
    printf 'unexpected top-level header link target for %s\n' "$top_level" >&2
    exit 1
  fi

  if ! cmp -s "$baseline" "$top_level"; then
    printf 'top-level header content mismatch: %s\n' "$top_level" >&2
    exit 1
  fi
done

if [[ "$(readlink "$stage_root/usr/include/libpng")" != "libpng16" ]]; then
  printf 'unexpected /usr/include/libpng compatibility symlink target\n' >&2
  exit 1
fi

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

if rg -q 'APNG' "$safe_dir/include/pnglibconf.h"; then
  printf 'pnglibconf.h unexpectedly references APNG support\n' >&2
  exit 1
fi

printf 'header baseline matches the exact staged header contract\n'
