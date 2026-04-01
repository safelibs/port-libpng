#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
package_version="$(dpkg-parsechangelog -l "$safe_dir/debian/changelog" -SVersion)"
output_path="$repo_root/libpng1.6_${package_version}.tar.xz"
manifest="$(mktemp)"
filtered_manifest="$(mktemp)"
tmp_output="$(mktemp --tmpdir "$package_version.XXXXXX.tar.xz")"

cleanup() {
  rm -f "$manifest" "$filtered_manifest" "$tmp_output"
}
trap cleanup EXIT

git -C "$repo_root" ls-files -z -- safe > "$manifest"
while IFS= read -r -d '' rel; do
  if [[ -e "$repo_root/$rel" || -L "$repo_root/$rel" ]]; then
    printf '%s\0' "$rel" >> "$filtered_manifest"
  fi
done < "$manifest"

if [[ ! -s "$filtered_manifest" ]]; then
  printf 'unable to build %s: git ls-files returned no tracked safe paths\n' "$output_path" >&2
  exit 1
fi

tar \
  -C "$repo_root" \
  --null \
  --verbatim-files-from \
  --files-from "$filtered_manifest" \
  -cJf "$tmp_output"

mv "$tmp_output" "$output_path"
