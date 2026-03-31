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

shared_lib="$(find "$stage_root/usr/lib" -name 'libpng16.so.16.43.0' -print -quit)"
static_lib="$(find "$stage_root/usr/lib" -name 'libpng16.a' -print -quit)"
if [[ -z "$shared_lib" || -z "$static_lib" ]]; then
  printf 'unable to locate staged safe libpng shared/static libraries under %s\n' "$stage_root/usr/lib" >&2
  exit 1
fi

lib_dir="$(dirname "$shared_lib")"
pkg_dir="$lib_dir/pkgconfig"
include_dir="$stage_root/usr/include"
bin_dir="$stage_root/usr/bin"
pkg_env="${PKG_CONFIG_PATH:-}"

for path in \
  "$include_dir/png.h" \
  "$include_dir/pngconf.h" \
  "$include_dir/pnglibconf.h" \
  "$include_dir/libpng16" \
  "$include_dir/libpng" \
  "$pkg_dir/libpng.pc" \
  "$pkg_dir/libpng16.pc" \
  "$bin_dir/libpng-config" \
  "$bin_dir/libpng16-config" \
  "$lib_dir/libpng16.so.16.43.0" \
  "$lib_dir/libpng16.so.16" \
  "$lib_dir/libpng16.so" \
  "$lib_dir/libpng.so" \
  "$lib_dir/libpng16.a" \
  "$lib_dir/libpng.a"
do
  if [[ ! -e "$path" && ! -L "$path" ]]; then
    printf 'missing staged install artifact: %s\n' "$path" >&2
    exit 1
  fi
done

if [[ "$(readlink "$include_dir/libpng")" != "libpng16" ]]; then
  printf 'unexpected /usr/include/libpng symlink target\n' >&2
  exit 1
fi

for header in png.h pngconf.h pnglibconf.h; do
  if [[ "$(readlink "$include_dir/$header")" != "libpng16/$header" ]]; then
    printf 'unexpected top-level header link target for %s\n' "$header" >&2
    exit 1
  fi
done

if [[ "$(readlink "$pkg_dir/libpng.pc")" != "libpng16.pc" ]]; then
  printf 'unexpected libpng.pc symlink target\n' >&2
  exit 1
fi

if [[ "$(readlink "$bin_dir/libpng-config")" != "libpng16-config" ]]; then
  printf 'unexpected libpng-config symlink target\n' >&2
  exit 1
fi

if [[ "$(readlink "$lib_dir/libpng16.so.16")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so.16 symlink target\n' >&2
  exit 1
fi

if [[ "$(readlink "$lib_dir/libpng16.so")" != "libpng16.so.16.43.0" ]]; then
  printf 'unexpected libpng16.so symlink target\n' >&2
  exit 1
fi

if [[ "$(readlink "$lib_dir/libpng.so")" != "libpng16.so" ]]; then
  printf 'unexpected libpng.so symlink target\n' >&2
  exit 1
fi

if [[ "$(readlink "$lib_dir/libpng.a")" != "libpng16.a" ]]; then
  printf 'unexpected libpng.a symlink target\n' >&2
  exit 1
fi

normalize_tokens() {
  tr '\n' ' ' | xargs
}

for package_name in libpng libpng16; do
  if ! PKG_CONFIG_PATH="$pkg_dir${pkg_env:+:$pkg_env}" pkg-config --exists "$package_name"; then
    printf 'pkg-config entry missing: %s\n' "$package_name" >&2
    exit 1
  fi

  cflags="$(PKG_CONFIG_PATH="$pkg_dir${pkg_env:+:$pkg_env}" pkg-config --cflags "$package_name" | normalize_tokens)"
  libs="$(PKG_CONFIG_PATH="$pkg_dir${pkg_env:+:$pkg_env}" pkg-config --libs "$package_name" | normalize_tokens)"
  static_libs="$(PKG_CONFIG_PATH="$pkg_dir${pkg_env:+:$pkg_env}" pkg-config --static --libs "$package_name" | normalize_tokens)"

  if [[ "$cflags" != "-I/usr/include/libpng16" ]]; then
    printf 'unexpected pkg-config cflags for %s: %s\n' "$package_name" "$cflags" >&2
    exit 1
  fi

  if [[ "$libs" != "-lpng16" ]]; then
    printf 'unexpected pkg-config libs for %s: %s\n' "$package_name" "$libs" >&2
    exit 1
  fi

  if [[ "$static_libs" != *"-lpng16"* || "$static_libs" != *"-lz"* ]]; then
    printf 'unexpected pkg-config static libs for %s: %s\n' "$package_name" "$static_libs" >&2
    exit 1
  fi
done

for config_script in "$bin_dir/libpng-config" "$bin_dir/libpng16-config"; do
  prefix="$("$config_script" --prefix)"
  version="$("$config_script" --version)"
  i_opts="$("$config_script" --I_opts | normalize_tokens)"
  libs="$("$config_script" --libs | normalize_tokens)"
  static_ldflags="$("$config_script" --static --ldflags | normalize_tokens)"

  if [[ "$prefix" != "/usr" ]]; then
    printf 'unexpected prefix from %s: %s\n' "$config_script" "$prefix" >&2
    exit 1
  fi

  if [[ "$version" != "1.6.43" ]]; then
    printf 'unexpected version from %s: %s\n' "$config_script" "$version" >&2
    exit 1
  fi

  if [[ "$i_opts" != "-I/usr/include/libpng16" ]]; then
    printf 'unexpected I_opts from %s: %s\n' "$config_script" "$i_opts" >&2
    exit 1
  fi

  if [[ "$libs" != "-lpng16" ]]; then
    printf 'unexpected libs from %s: %s\n' "$config_script" "$libs" >&2
    exit 1
  fi

  if [[ "$static_ldflags" != *"-lpng16"* || "$static_ldflags" != *"-lz"* ]]; then
    printf 'unexpected static ldflags from %s: %s\n' "$config_script" "$static_ldflags" >&2
    exit 1
  fi
done

set +e
libdir_output="$("$bin_dir/libpng-config" --libdir 2>&1)"
libdir_status=$?
set -e
if [[ $libdir_status -ne 1 ]]; then
  printf 'expected libpng-config --libdir to fail with status 1, got %d\n' "$libdir_status" >&2
  exit 1
fi

if [[ "$libdir_output" != "libpng-config: --libdir option is disabled in Debian/Ubuntu" ]]; then
  printf 'unexpected libpng-config --libdir output: %s\n' "$libdir_output" >&2
  exit 1
fi

printf 'install surface matches the staged Ubuntu-compatible libpng layout\n'
