#!/usr/bin/env bash
set -euo pipefail

script_path="$(readlink -f -- "${BASH_SOURCE[0]}")"
script_dir="$(cd -- "$(dirname -- "$script_path")" && pwd)"
safe_dir="$(cd -- "$script_dir/.." && pwd)"
repo_root="$(cd -- "$safe_dir/.." && pwd)"
pwd_real="$(pwd -P)"

if [[ "$pwd_real" != "$safe_dir" && "$pwd_real" != "$safe_dir/"* && "$pwd_real" != "$repo_root" && "$pwd_real" != "$repo_root/"* ]]; then
  exec /usr/bin/dpkg-genbuildinfo "$@"
fi

/usr/bin/dpkg-genbuildinfo "$@"
python3 "$script_dir/normalize-package-metadata.py" buildinfo "$@"
