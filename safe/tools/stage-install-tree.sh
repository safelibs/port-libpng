#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
profile="${PROFILE:-release}"
target_root="${CARGO_TARGET_DIR:-$safe_dir/target}"
profile_dir="$target_root/$profile"
stage_root="${STAGE_ROOT:-$profile_dir/abi-stage}"

static_lib="$profile_dir/libpng16.a"

if [[ ! -f "$static_lib" ]]; then
  printf 'missing built Rust outputs under %s; run cargo build first\n' "$profile_dir" >&2
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

render_template() {
  local template="$1"

  sed \
    -e 's#@prefix@#/usr#g' \
    -e 's#@exec_prefix@#${prefix}#g' \
    -e "s#@libdir@#/usr/lib/$multiarch#g" \
    -e 's#@includedir@#/usr/include#g' \
    -e 's#@PNGLIB_MAJOR@#1#g' \
    -e 's#@PNGLIB_MINOR@#6#g' \
    -e 's#@PNGLIB_VERSION@#1.6.43#g' \
    -e 's#@LIBS@#-lm -lz -lm #g' \
    "$template"
}

ensure_symlink() {
  local link_path="$1"
  local target="$2"

  rm -f "$link_path"
  ln -s "$target" "$link_path"
}

link_versioned_shared_library() {
  local output_path="$1"
  local cc_bin="${CC:-cc}"

  "$cc_bin" \
    -shared \
    -Wl,--whole-archive "$static_lib" -Wl,--no-whole-archive \
    -Wl,--version-script="$safe_dir/abi/libpng.vers" \
    -Wl,-soname,libpng16.so.16 \
    -lz -lm -ldl -lpthread -lrt -lutil -lgcc_s \
    -o "$output_path"
}

multiarch="$(detect_multiarch)"
include_root="$stage_root/usr/include"
include_subdir="$include_root/libpng16"
lib_root="$stage_root/usr/lib/$multiarch"
pkg_root="$lib_root/pkgconfig"
bin_root="$stage_root/usr/bin"

rm -rf "$stage_root"
mkdir -p "$include_subdir" "$pkg_root" "$bin_root"

for header in png.h pngconf.h pnglibconf.h; do
  install -m 0644 "$safe_dir/include/$header" "$include_subdir/$header"
  ensure_symlink "$include_root/$header" "libpng16/$header"
done
ensure_symlink "$include_root/libpng" "libpng16"

render_template "$safe_dir/pkg/libpng.pc.in" > "$pkg_root/libpng16.pc"
ensure_symlink "$pkg_root/libpng.pc" "libpng16.pc"

render_template "$safe_dir/pkg/libpng-config.in" > "$bin_root/libpng16-config"
chmod 0755 "$bin_root/libpng16-config"
ensure_symlink "$bin_root/libpng-config" "libpng16-config"

link_versioned_shared_library "$lib_root/libpng16.so.16.43.0"
ensure_symlink "$lib_root/libpng16.so.16" "libpng16.so.16.43.0"
ensure_symlink "$lib_root/libpng16.so" "libpng16.so.16.43.0"
ensure_symlink "$lib_root/libpng.so" "libpng16.so"

install -m 0644 "$static_lib" "$lib_root/libpng16.a"
ensure_symlink "$lib_root/libpng.a" "libpng16.a"
