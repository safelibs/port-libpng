#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
source "$safe_dir/tests/upstream/common.sh"

mapfile -t upstream_wrappers < <(extract_upstream_tests)

if [[ "${#upstream_wrappers[@]}" -eq 0 ]]; then
  printf 'failed to enumerate upstream smoke cases from the in-tree package inputs\n' >&2
  exit 1
fi

if printf '%s\n' "${upstream_wrappers[@]}" | grep -qx 'pngstest'; then
  printf 'upstream tests/pngstest must be treated as a helper, not a standalone test case\n' >&2
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

wrapper_jobs="${LIBPNG_SAFE_UPSTREAM_JOBS:-$(build_jobs)}"
if [[ "$wrapper_jobs" -gt 12 ]]; then
  wrapper_jobs=12
fi

printf '%s\n' "${upstream_wrappers[@]}" | xargs -P"$wrapper_jobs" -I{} \
  bash -lc '
    set -euo pipefail
    source "'"$safe_dir/tests/upstream/common.sh"'"
    build_dir="$(mktemp -d)"
    log_file="$(mktemp)"
    trap "rm -rf \"$build_dir\" \"$log_file\"" EXIT

    if run_wrapper_case "$1" "$build_dir" >"$log_file" 2>&1; then
      printf "PASS: %s\n" "$1"
    else
      printf "FAIL: %s\n" "$1" >&2
      cat "$log_file" >&2
      exit 1
    fi
  ' _ {}

for smoke_script in pngcp.sh timepng.sh pngfix.sh; do
  printf '==> %s\n' "${smoke_script%.sh}"
  "$safe_dir/tests/upstream/$smoke_script"
done

printf '==> %s\n' "png-fix-itxt"
"$safe_dir/tests/upstream/png-fix-itxt.sh"

printf 'upstream smoke matrix passed against the staged safe libpng build\n'
printf 'explicit consumer smokes passed for pngcp, timepng, and pngfix\n'
printf 'standalone packaged-tool smoke passed for png-fix-itxt\n'
