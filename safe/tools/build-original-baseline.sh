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

  printf 'unable to determine multiarch libdir: need LIBPNG_MULTIARCH, dpkg-architecture, or gcc\n' >&2
  exit 1
}

build_jobs() {
  if command -v nproc >/dev/null 2>&1; then
    nproc
    return 0
  fi

  getconf _NPROCESSORS_ONLN 2>/dev/null || printf '1\n'
}

update_file_if_changed() {
  local source="$1"
  local target="$2"

  if [[ -f "$target" ]] && cmp -s "$source" "$target"; then
    printf 'unchanged %s\n' "$target"
    return 0
  fi

  install -m 0644 "$source" "$target"
  printf 'updated %s\n' "$target"
}

multiarch="$(detect_multiarch)"
build_dir="$(mktemp -d)"
install_root="$(mktemp -d)"
generated_pnglibconf="$(mktemp)"
generated_version_script="$(mktemp)"
generated_exports="$(mktemp)"
generated_install_layout="$(mktemp)"

cleanup() {
  rm -rf "$build_dir" "$install_root"
  rm -f \
    "$generated_pnglibconf" \
    "$generated_version_script" \
    "$generated_exports" \
    "$generated_install_layout"
}

trap cleanup EXIT

mkdir -p "$safe_dir/abi" "$safe_dir/include"

(
  cd "$build_dir"
  "$original_dir/configure" \
    --prefix=/usr \
    --libdir="/usr/lib/$multiarch" \
    --includedir=/usr/include \
    --enable-shared \
    --enable-static \
    --enable-tools \
    --disable-silent-rules
  make -j"$(build_jobs)"
  make install DESTDIR="$install_root"
)

install -m 0644 "$install_root/usr/include/libpng16/pnglibconf.h" "$generated_pnglibconf"
install -m 0644 "$build_dir/libpng.vers" "$generated_version_script"

nm -D --defined-only "$build_dir/.libs/libpng16.so.16.43.0" \
  | awk '{print $3}' \
  | sed 's/@.*$//' \
  | grep '^png_' \
  | LC_ALL=C sort -u \
  > "$generated_exports"

find "$install_root" \( -type f -o -type l \) -printf '%P\n' \
  | LC_ALL=C sort \
  > "$generated_install_layout"

export_count="$(wc -l < "$generated_exports" | tr -d ' ')"
if [[ "$export_count" != "246" ]]; then
  printf 'unexpected export count: expected 246, found %s\n' "$export_count" >&2
  exit 1
fi

if grep -qx 'png_err' "$generated_exports"; then
  printf 'png_err unexpectedly present in frozen export list\n' >&2
  exit 1
fi

if grep -qx 'png_set_strip_error_numbers' "$generated_exports"; then
  printf 'png_set_strip_error_numbers unexpectedly present in frozen export list\n' >&2
  exit 1
fi

update_file_if_changed "$generated_pnglibconf" "$safe_dir/include/pnglibconf.h"
update_file_if_changed "$generated_version_script" "$safe_dir/abi/libpng.vers"
update_file_if_changed "$generated_exports" "$safe_dir/abi/exports.txt"
update_file_if_changed "$generated_install_layout" "$safe_dir/abi/install-layout.txt"
