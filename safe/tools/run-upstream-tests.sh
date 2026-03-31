#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
source "$safe_dir/tests/upstream/common.sh"

mapfile -t upstream_wrappers < <(extract_upstream_tests)

if [[ "${#upstream_wrappers[@]}" -eq 0 ]]; then
  printf 'failed to extract upstream TESTS list from %s\n' "$repo_root/original/Makefile.am" >&2
  exit 1
fi

if printf '%s\n' "${upstream_wrappers[@]}" | grep -qx 'pngstest'; then
  printf 'original/tests/pngstest must be treated as a helper, not a standalone test case\n' >&2
  exit 1
fi

declare -A build_dirs=()
cleanup_dirs=()

cleanup() {
  local dir

  for dir in "${cleanup_dirs[@]}"; do
    rm -rf "$dir"
  done
}

trap cleanup EXIT

ensure_safe_stage

for wrapper_name in "${upstream_wrappers[@]}"; do
  program="$(wrapper_program_for "$wrapper_name")"

  if [[ -z "${build_dirs[$program]:-}" ]]; then
    build_dirs[$program]="$(mktemp -d)"
    cleanup_dirs+=("${build_dirs[$program]}")
    compile_wrapper_program "$program" "${build_dirs[$program]}"
  fi

  printf '==> %s\n' "$wrapper_name"
  run_original_wrapper "$wrapper_name" "${build_dirs[$program]}"
done

for smoke_script in pngcp.sh timepng.sh pngfix.sh; do
  printf '==> %s\n' "${smoke_script%.sh}"
  "$safe_dir/tests/upstream/$smoke_script"
done

printf '==> %s\n' "png-fix-itxt"
"$safe_dir/tests/upstream/png-fix-itxt.sh"

printf 'upstream wrapper matrix passed against the staged safe libpng build\n'
printf 'explicit consumer smokes passed for pngcp, timepng, and pngfix\n'
printf 'standalone packaged-tool smoke passed for png-fix-itxt\n'
