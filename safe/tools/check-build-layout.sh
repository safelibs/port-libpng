#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
stage_root="${STAGE_ROOT:-$target_root/$profile/abi-stage}"
layout_baseline="$safe_dir/abi/install-layout.txt"

build_args=(build --manifest-path "$safe_dir/Cargo.toml")
if [[ "$profile" == "release" ]]; then
  build_args+=(--release)
else
  build_args+=(--profile "$profile")
fi

cargo "${build_args[@]}"
"$safe_dir/tools/stage-install-tree.sh"

mapfile -t required_paths < <(
  grep -E \
    '(^usr/bin/libpng(16)?-config$)|(^usr/include/libpng$)|(^usr/include/(libpng16/)?png(libconf|conf)?\.h$)|(^usr/lib/.*/(libpng16\.so\.16\.43\.0|libpng16\.so\.16|libpng16\.so|libpng16\.a|libpng\.so|libpng\.a)$)|(^usr/lib/.*/pkgconfig/libpng(16)?\.pc$)' \
    "$layout_baseline"
)

for rel in "${required_paths[@]}"; do
  path="$stage_root/$rel"
  if [[ ! -e "$path" && ! -L "$path" ]]; then
    printf 'missing staged install path: %s\n' "$path" >&2
    exit 1
  fi
done

libdir_rel="$(grep -E '^usr/lib/.*/libpng16\.so\.16\.43\.0$' "$layout_baseline" | sed 's#/libpng16\.so\.16\.43\.0$##')"
libdir="$stage_root/$libdir_rel"

for staged_file in "$libdir/libpng16.so.16.43.0" "$libdir/libpng16.a"; do
  if [[ ! -f "$staged_file" || -L "$staged_file" ]]; then
    printf 'staged install artifact must be a regular file: %s\n' "$staged_file" >&2
    exit 1
  fi

  if [[ "$(readlink -f "$staged_file")" != "$staged_file" ]]; then
    printf 'staged install artifact resolves outside the staged tree: %s\n' "$staged_file" >&2
    exit 1
  fi
done

if [[ "$(readlink "$libdir/libpng16.so.16")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so.16 link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng16.so")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng.so")" != "libpng16.so" ]]; then
  printf 'unexpected libpng.so link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/libpng.a")" != "libpng16.a" ]]; then
  printf 'unexpected libpng.a link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$libdir/pkgconfig/libpng.pc")" != "libpng16.pc" ]]; then
  printf 'unexpected libpng.pc link target\n' >&2
  exit 1
fi

if [[ "$(readlink "$stage_root/usr/bin/libpng-config")" != "libpng16-config" ]]; then
  printf 'unexpected libpng-config link target\n' >&2
  exit 1
fi

for header in png.h pngconf.h pnglibconf.h; do
  if [[ "$(readlink "$stage_root/usr/include/$header")" != "libpng16/$header" ]]; then
    printf 'unexpected top-level header link target for %s\n' "$header" >&2
    exit 1
  fi
done

if [[ "$(readlink "$stage_root/usr/include/libpng")" != "libpng16" ]]; then
  printf 'unexpected include compatibility symlink for %s\n' "$stage_root/usr/include/libpng" >&2
  exit 1
fi

printf 'bootstrap install layout matches the frozen subset baseline\n'
