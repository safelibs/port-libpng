#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
source "$safe_dir/tests/upstream/common.sh"

build_dir="$(mktemp -d)"
trap 'rm -rf "$build_dir"' EXIT

ensure_safe_stage

build_pngcp_consumer "$build_dir"
build_pngfix_consumer "$build_dir"
build_timepng_consumer "$build_dir"
compile_libpng_client pngtopng "$safe_dir/tests/upstream/pngtopng.c" "$build_dir"
build_png_fix_itxt_tool "$build_dir"

smoke_pngcp "$build_dir"
smoke_pngfix "$build_dir"
smoke_timepng "$build_dir"

pngtopng_output="$build_dir/pngtopng-output.png"
"$build_dir/pngtopng" \
  "$repo_root/original/pngtest.png" \
  "$pngtopng_output"

if [[ ! -s "$pngtopng_output" ]]; then
  printf 'pngtopng did not produce an output file\n' >&2
  exit 1
fi

"$build_dir/timepng" "$pngtopng_output" >/dev/null
smoke_png_fix_itxt "$build_dir"

printf 'libpng consumer smokes passed for pngcp, pngfix, timepng, and pngtopng (simplified write)\n'
printf 'standalone packaged-tool smoke passed separately for png-fix-itxt\n'
