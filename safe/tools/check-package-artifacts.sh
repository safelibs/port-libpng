#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"

latest_artifact() {
  local package_name="$1"
  local extension="$2"

  find "$repo_root" -maxdepth 1 -type f -name "${package_name}_*.${extension}" -printf '%T@ %p\n' \
    | sort -nr \
    | head -n1 \
    | cut -d' ' -f2-
}

require_artifact() {
  local package_name="$1"
  local extension="$2"
  local artifact

  artifact="$(latest_artifact "$package_name" "$extension")"
  if [[ -z "$artifact" ]]; then
    printf 'missing built package artifact: %s_*.%s\n' "$package_name" "$extension" >&2
    exit 1
  fi

  printf '%s\n' "$artifact"
}

require_path() {
  local root="$1"
  local rel="$2"

  if [[ ! -e "$root/$rel" && ! -L "$root/$rel" ]]; then
    printf 'missing packaged path: %s\n' "$rel" >&2
    exit 1
  fi
}

require_symlink_target() {
  local root="$1"
  local rel="$2"
  local expected="$3"

  require_path "$root" "$rel"
  if [[ "$(readlink "$root/$rel")" != "$expected" ]]; then
    printf 'unexpected symlink target for %s: expected %s, found %s\n' \
      "$rel" "$expected" "$(readlink "$root/$rel")" >&2
    exit 1
  fi
}

runtime_deb="$(require_artifact libpng16-16t64 deb)"
dev_deb="$(require_artifact libpng-dev deb)"
tools_deb="$(require_artifact libpng-tools deb)"
udeb_artifact="$(require_artifact libpng16-16-udeb udeb)"

for pair in \
  "libpng16-16t64:$runtime_deb" \
  "libpng-dev:$dev_deb" \
  "libpng-tools:$tools_deb" \
  "libpng16-16-udeb:$udeb_artifact"
do
  expected_name="${pair%%:*}"
  artifact="${pair#*:}"
  actual_name="$(dpkg-deb -f "$artifact" Package)"

  if [[ "$actual_name" != "$expected_name" ]]; then
    printf 'unexpected package name in %s: expected %s, found %s\n' \
      "$artifact" "$expected_name" "$actual_name" >&2
    exit 1
  fi
done

runtime_version="$(dpkg-deb -f "$runtime_deb" Version)"
for artifact in "$dev_deb" "$tools_deb" "$udeb_artifact"; do
  if [[ "$(dpkg-deb -f "$artifact" Version)" != "$runtime_version" ]]; then
    printf 'package version mismatch: expected %s in %s\n' "$runtime_version" "$artifact" >&2
    exit 1
  fi
done

extract_root="$(mktemp -d)"
trap 'rm -rf "$extract_root"' EXIT

runtime_root="$extract_root/runtime"
dev_root="$extract_root/dev"
tools_root="$extract_root/tools"
udeb_root="$extract_root/udeb"

dpkg-deb -x "$runtime_deb" "$runtime_root"
dpkg-deb -x "$dev_deb" "$dev_root"
dpkg-deb -x "$tools_deb" "$tools_root"
dpkg-deb -x "$udeb_artifact" "$udeb_root"

multiarch_dir="$(find "$runtime_root/usr/lib" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | head -n1)"
if [[ -z "$multiarch_dir" ]]; then
  printf 'unable to determine multiarch runtime directory from %s\n' "$runtime_deb" >&2
  exit 1
fi

require_path "$runtime_root" "usr/lib/$multiarch_dir/libpng16.so.16.43.0"
require_symlink_target "$runtime_root" "usr/lib/$multiarch_dir/libpng16.so.16" "libpng16.so.16.43.0"

runtime_lib="$runtime_root/usr/lib/$multiarch_dir/libpng16.so.16.43.0"
for symbol_name in \
  png_destroy_read_struct \
  png_set_expand \
  png_set_gray_to_rgb \
  png_set_interlace_handling \
  png_set_palette_to_rgb \
  png_set_strip_16 \
  png_set_tRNS_to_alpha
do
  if ! objdump -T "$runtime_lib" | grep -Eq "[[:space:]]PNG16_0[[:space:]]+$symbol_name$"; then
    printf 'runtime package is missing PNG16_0 versioning for %s\n' "$symbol_name" >&2
    exit 1
  fi
done

for rel in \
  "usr/include/libpng16/png.h" \
  "usr/include/libpng16/pngconf.h" \
  "usr/include/libpng16/pnglibconf.h" \
  "usr/include/png.h" \
  "usr/include/pngconf.h" \
  "usr/include/pnglibconf.h" \
  "usr/include/libpng" \
  "usr/lib/$multiarch_dir/libpng16.a" \
  "usr/lib/$multiarch_dir/libpng.a" \
  "usr/lib/$multiarch_dir/libpng16.so" \
  "usr/lib/$multiarch_dir/libpng.so" \
  "usr/lib/$multiarch_dir/pkgconfig/libpng16.pc" \
  "usr/lib/$multiarch_dir/pkgconfig/libpng.pc" \
  "usr/bin/libpng16-config" \
  "usr/bin/libpng-config" \
  "usr/share/man/man1/libpng16-config.1.gz" \
  "usr/share/man/man1/libpng-config.1.gz" \
  "usr/share/man/man3/libpng.3.gz" \
  "usr/share/man/man5/png.5.gz"
do
  require_path "$dev_root" "$rel"
done

require_symlink_target "$dev_root" "usr/include/libpng" "libpng16"
require_symlink_target "$dev_root" "usr/include/png.h" "libpng16/png.h"
require_symlink_target "$dev_root" "usr/include/pngconf.h" "libpng16/pngconf.h"
require_symlink_target "$dev_root" "usr/include/pnglibconf.h" "libpng16/pnglibconf.h"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/libpng.a" "libpng16.a"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/libpng.so" "libpng16.so"
require_symlink_target "$dev_root" "usr/lib/$multiarch_dir/pkgconfig/libpng.pc" "libpng16.pc"
require_symlink_target "$dev_root" "usr/bin/libpng-config" "libpng16-config"

examples_dir="$dev_root/usr/share/doc/libpng-dev/examples"
for example_name in example.c pngtest.c pngtest.png; do
  require_path "$dev_root" "usr/share/doc/libpng-dev/examples/$example_name"
done

for example_name in example.c pngtest.c; do
  if ! cmp -s "$examples_dir/$example_name" "$repo_root/original/$example_name"; then
    printf 'packaged libpng-dev example payload diverged: %s\n' "$example_name" >&2
    exit 1
  fi
done

pngtest_description="$(file -b "$examples_dir/pngtest.png")"
if [[ "$pngtest_description" != *"PNG image data, 91 x 69"* ]]; then
  printf 'unexpected packaged pngtest.png description: %s\n' "$pngtest_description" >&2
  exit 1
fi

for rel in \
  "usr/bin/pngfix" \
  "usr/bin/png-fix-itxt"
do
  require_path "$tools_root" "$rel"
done

udeb_lib="$(find "$udeb_root/lib" -name 'libpng16.so.16.43.0' -print -quit)"
if [[ -z "$udeb_lib" ]]; then
  printf 'udeb payload is missing libpng16.so.16.43.0\n' >&2
  exit 1
fi

udeb_lib_dir="$(dirname "$udeb_lib")"
if [[ "$(readlink "$udeb_lib_dir/libpng16.so.16")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected udeb soname symlink target\n' >&2
  exit 1
fi

printf 'package artifacts passed for %s\n' "$runtime_version"
printf 'libpng-dev examples preserved under /usr/share/doc/libpng-dev/examples/\n'
