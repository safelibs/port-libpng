#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
original_dir="$repo_root/original"

if [[ ! -x "$original_dir/configure" ]]; then
  printf 'missing original configure script at %s\n' "$original_dir/configure" >&2
  exit 1
fi

if command -v dpkg-architecture >/dev/null 2>&1; then
  multiarch="$(dpkg-architecture -qDEB_HOST_MULTIARCH)"
elif command -v gcc >/dev/null 2>&1; then
  multiarch="$(gcc -print-multiarch)"
else
  printf 'unable to determine multiarch libdir: need dpkg-architecture or gcc\n' >&2
  exit 1
fi

if [[ -z "$multiarch" ]]; then
  printf 'failed to determine a non-empty multiarch triplet\n' >&2
  exit 1
fi

build_dir="$(mktemp -d)"
install_root="$(mktemp -d)"

cleanup() {
  rm -rf "$build_dir" "$install_root"
}

trap cleanup EXIT

mkdir -p "$safe_dir/abi" "$safe_dir/include"

cd "$build_dir"
"$original_dir/configure" \
  --prefix=/usr \
  --libdir="/usr/lib/$multiarch" \
  --includedir=/usr/include \
  --enable-shared \
  --enable-static \
  --enable-tools \
  --disable-silent-rules
make -j"$(nproc)"
make install DESTDIR="$install_root"

install -m 0644 "$install_root/usr/include/libpng16/pnglibconf.h" \
  "$safe_dir/include/pnglibconf.h"
install -m 0644 "$build_dir/libpng.vers" \
  "$safe_dir/abi/libpng.vers"

nm -D --defined-only "$build_dir/.libs/libpng16.so.16.43.0" \
  | awk '{print $3}' \
  | sed 's/@.*$//' \
  | grep '^png_' \
  | LC_ALL=C sort -u \
  > "$safe_dir/abi/exports.txt"

find "$install_root" \( -type f -o -type l \) -printf '%P\n' \
  | LC_ALL=C sort \
  > "$safe_dir/abi/install-layout.txt"

export_count="$(wc -l < "$safe_dir/abi/exports.txt" | tr -d ' ')"
if [[ "$export_count" != "246" ]]; then
  printf 'unexpected export count: expected 246, found %s\n' "$export_count" >&2
  exit 1
fi

if grep -qx 'png_err' "$safe_dir/abi/exports.txt"; then
  printf 'png_err unexpectedly present in frozen export list\n' >&2
  exit 1
fi

if grep -qx 'png_set_strip_error_numbers' "$safe_dir/abi/exports.txt"; then
  printf 'png_set_strip_error_numbers unexpectedly present in frozen export list\n' >&2
  exit 1
fi

printf 'wrote %s\n' "$safe_dir/include/pnglibconf.h"
printf 'wrote %s\n' "$safe_dir/abi/libpng.vers"
printf 'wrote %s\n' "$safe_dir/abi/exports.txt"
printf 'wrote %s\n' "$safe_dir/abi/install-layout.txt"
