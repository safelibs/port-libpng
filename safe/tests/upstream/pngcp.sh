#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
source "$script_dir/common.sh"

build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

build_pngcp_consumer "$build_dir"
smoke_pngcp "$build_dir"
