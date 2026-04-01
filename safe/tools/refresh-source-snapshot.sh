#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source_root="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$source_root/.." && pwd)"
package_version="$(dpkg-parsechangelog -l "$source_root/debian/changelog" -SVersion)"
output_path="$repo_root/libpng1.6_${package_version}.tar.xz"
manifest="$source_root/pkg/source-snapshot-manifest.txt"
filtered_manifest="$(mktemp)"
tmp_output="$(mktemp --tmpdir "$package_version.XXXXXX.tar.xz")"
source_date_epoch="${SOURCE_DATE_EPOCH:-$(dpkg-parsechangelog -l "$source_root/debian/changelog" -STimestamp)}"

cleanup() {
  rm -f "$filtered_manifest" "$tmp_output"
}
trap cleanup EXIT

if [[ ! -f "$manifest" ]]; then
  printf 'unable to build %s: missing source snapshot manifest %s\n' "$output_path" "$manifest" >&2
  exit 1
fi

while IFS= read -r rel; do
  if [[ -z "$rel" ]]; then
    continue
  fi
  if [[ -e "$source_root/$rel" || -L "$source_root/$rel" ]]; then
    printf '%s\0' "$rel" >> "$filtered_manifest"
  fi
done < "$manifest"

if [[ ! -s "$filtered_manifest" ]]; then
  printf 'unable to build %s: source snapshot manifest %s produced no existing paths\n' "$output_path" "$manifest" >&2
  exit 1
fi

tar \
  -C "$source_root" \
  --null \
  --sort=name \
  --mtime="@$source_date_epoch" \
  --owner=0 \
  --group=0 \
  --numeric-owner \
  --transform='s,^,safe/,' \
  --verbatim-files-from \
  --files-from "$filtered_manifest" \
  -cJf "$tmp_output"

mv "$tmp_output" "$output_path"
