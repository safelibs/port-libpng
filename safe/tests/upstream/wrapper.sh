#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'usage: %s <upstream-wrapper>\n' "${0##*/}" >&2
  exit 1
fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

run_wrapper_case "$1" "$build_dir"
