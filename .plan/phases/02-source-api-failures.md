# Phase Name

Fix source/API validator failures

# Implement Phase ID

`impl-source-api-validator-failures`

# Preexisting Inputs

Consume prior artifacts in place.

- `validator/artifacts/libpng-safe-initial/` from Phase 1
- `validator/artifacts/libpng-safe-initial/results/libpng/`
- `validator/artifacts/libpng-safe-initial/logs/libpng/`
- `validator-case-inventory.json`
- `validator-report.md`
- Current `validator/` checkout
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing override packages under `validator-overrides/libpng/`
- Existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`
- Source/API validator scripts when present:
  - `validator/tests/libpng/tests/cases/source/chunk-metadata-inspection.sh`
  - `validator/tests/libpng/tests/cases/source/read-write-c-api-smoke.sh`
- Safe source modules:
  - `safe/src/read.rs`
  - `safe/src/write.rs`
  - `safe/src/write_runtime.rs`
  - `safe/src/get.rs`
  - `safe/src/set.rs`
  - `safe/src/state.rs`
  - `safe/src/bridge_ffi.rs`
  - `safe/src/compat_exports.rs`
- Local C test directories:
  - `safe/tests/core-smoke/`
  - `safe/tests/read-core/`
  - `safe/tests/read-transforms/`

# New Outputs

- Minimal source/API regression tests in `safe/tests/core-smoke/`, `safe/tests/read-core/`, or `safe/tests/read-transforms/`
- Safe code fixes in the smallest relevant module set
- Rebuilt packages and refreshed `validator-overrides/libpng/*.deb`
- Full validator rerun artifacts under `validator/artifacts/libpng-safe-source-api/`, produced by this phase even when no source/API failures exist
- Updated source/API section in `validator-report.md`
- A git commit, for example `validator: fix libpng source API regressions`, or a documentation-only commit if no source/API failures exist

# File Changes

- Possible tests:
  - `safe/tests/core-smoke/validator_read_write_c_api.c`
  - `safe/tests/read-core/validator_chunk_metadata.c`
  - Other targeted `safe/tests/core-smoke/`, `safe/tests/read-core/`, or `safe/tests/read-transforms/` files
- Candidate source fixes:
  - `safe/src/read.rs`
  - `safe/src/write.rs`
  - `safe/src/write_runtime.rs`
  - `safe/src/get.rs`
  - `safe/src/set.rs`
  - `safe/src/state.rs`
  - `safe/src/bridge_ffi.rs`
  - `safe/src/compat_exports.rs`
- `validator-report.md`
- Package artifacts and `validator-overrides/libpng/` as needed after rebuild
- Do not edit validator testcase scripts to make failures pass.

# Implementation Details

1. Inspect Phase 1 per-case JSON and logs for `chunk-metadata-inspection` and `read-write-c-api-smoke`.
2. If both cases passed in the baseline, document "no source/API failures in current run", rebuild packages, refresh `validator-overrides/libpng/`, rerun the full validator to `validator/artifacts/libpng-safe-source-api/`, verify, clean Debian build products, run the commit gate, commit, and continue.
3. For each failing source/API case, add the smallest local C reproducer. Use `safe/tests/core-smoke/` for pure ABI smoke tests and `safe/tests/read-core/` only if the driver is already or can be safely wired into `check-read-core.sh`.
4. `chunk-metadata-inspection` must compile against staged headers, read a PNGSuite fixture, and assert width, height, and color type getters.
5. `read-write-c-api-smoke` must write a 1x1 RGB PNG through `png_write_info`, `png_write_image`, and `png_write_end`, then read it back through `png_read_info` and getters.
6. Fix the compatibility issue in `safe/src/` without special-casing validator paths or fixture names.
7. Preserve libpng error semantics, existing `abi_guard!` usage, callback and `setjmp`/`longjmp` behavior, and stable Rust-owned metadata storage in `PngInfoState`.
8. Run the verifier local command battery.
9. Rebuild packages, refresh `validator-overrides/libpng/`, remove stale `validator/artifacts/libpng-safe-source-api/`, and run the full libpng validator from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-source-api" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-source-api/validator.exit-code`.
10. Update `validator-report.md` with reproducer paths, source files changed, validator summary, and remaining failures by class.
11. Remove generated Debian build directories and files, run the Debian build-product commit gate below, and commit before yielding.

## Required Debian build-product cleanup

Run this before the Debian build-product commit gate and before committing:

```bash
rm -rf \
  safe/debian/.debhelper \
  safe/debian/tmp \
  safe/debian/libpng-dev \
  safe/debian/libpng-tools \
  safe/debian/libpng16-16-udeb \
  safe/debian/libpng16-16t64 \
  safe/debian/build-tools \
  safe/debian/cargo-home \
  safe/debian/upstream-source-root
rm -f \
  safe/debian/*.debhelper.log \
  safe/debian/*.substvars \
  safe/debian/files \
  safe/debian/debhelper-build-stamp
```

# Critical Files And Workflow Constraints

- Do not modify `.plan/plan.md`.
- Treat `safe/Cargo.toml` and `safe/Cargo.lock` as critical dependency manifests; avoid dependency changes unless unavoidable and documented.
- Consume Phase 1 artifacts in place, especially `validator/artifacts/libpng-safe-initial/`, `validator-case-inventory.json`, `validator-report.md`, root package artifacts, `validator-overrides/libpng/*.deb`, and the existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`.
- The source/API validator scripts are reproduction inputs only: `validator/tests/libpng/tests/cases/source/chunk-metadata-inspection.sh` and `validator/tests/libpng/tests/cases/source/read-write-c-api-smoke.sh` must not be edited.
- Validator suite files are read-only except for clone or fast-forward updates made in Phase 1. Review with `git -C validator status --short` and `git -C validator diff -- tests tools repositories.yml test.sh`.
- Keep source/API fixes in `safe/` unless a validator bug is proven and documented with full failing evidence in `validator-report.md`.
- Treat `safe/include/png.h`, `safe/include/pngconf.h`, `safe/include/pnglibconf.h`, `safe/abi/exports.txt`, `safe/abi/libpng.vers`, and `safe/abi/install-layout.txt` as critical ABI/install baselines; change them only for a documented compatibility reason and verify with `check-headers.sh`, `check-exports.sh`, and related install-surface checks.
- Every implementation phase must rebuild packages, refresh `validator-overrides/libpng/`, run a fresh full-suite validator root, write `validator.exit-code`, update `validator-report.md`, run the Debian build-product cleanup and commit gate, and commit before yielding.

# Verification Phases

## `check-source-api-validator-failures-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-source-api-validator-failures`
- Purpose: confirm source/API failures have local regressions and pass after the fix.
- Commands:
  - `cargo fmt --check --manifest-path safe/Cargo.toml`
  - `cargo test --manifest-path safe/Cargo.toml`
  - `safe/tools/check-core-smoke.sh`
  - `safe/tools/check-read-core.sh`
  - `safe/tools/check-read-transforms.sh`
  - `safe/tools/check-exports.sh`
  - `safe/tools/check-headers.sh`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Run this source/API result check:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    inventory = json.loads(Path("validator-case-inventory.json").read_text())
    expected = [
        case_id
        for case_id in ("chunk-metadata-inspection", "read-write-c-api-smoke")
        if case_id in set(inventory.get("source_case_ids", []))
    ]
    root = Path("validator/artifacts/libpng-safe-source-api")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"missing source/API summary: {summary_path}")
    summary = json.loads(summary_path.read_text())
    if summary.get("library") != "libpng" or summary.get("mode") != "original":
        raise SystemExit(f"unexpected source/API summary: {summary}")
    failures = []
    for case_id in expected:
        path = results_dir / f"{case_id}.json"
        if not path.is_file():
            failures.append(f"{case_id}: missing result JSON")
            continue
        result = json.loads(path.read_text())
        if result.get("status") != "passed" or result.get("exit_code") != 0:
            failures.append(f"{case_id}: status={result.get('status')} exit_code={result.get('exit_code')} log={result.get('log_path')}")
        if result.get("override_debs_installed") is not True:
            failures.append(f"{case_id}: override_debs_installed is not true")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"source/API cases passed in {root}: {', '.join(expected) if expected else 'none in inventory'}")
    PY
    ```

  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-source-api`.
  - Inline and run the Debian build-product commit gate below.

## `check-source-api-validator-failures-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-source-api-validator-failures`
- Purpose: review correctness and minimality of source/API changes.
- Commands:
  - `git show --stat --oneline HEAD`
  - `git diff HEAD~1..HEAD -- safe/src safe/tests validator-report.md`
  - `test -z "$(git -C validator status --short -- tests tools repositories.yml test.sh)"`
  - `git -C validator diff --exit-code -- tests tools repositories.yml test.sh`
  - `rg -n "chunk-metadata-inspection|read-write-c-api-smoke|source/API|source-api" validator-report.md`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-source-api`.
  - Inline and run the Debian build-product commit gate below.
  - Run this artifact presence check:

    ```bash
    python3 - <<'PY'
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-source-api")
    required = [
        root / "validator.exit-code",
        root / "results/libpng/summary.json",
        root / "logs/libpng",
        root / "casts/libpng",
    ]
    missing = [str(path) for path in required if not path.exists()]
    if missing:
        raise SystemExit("missing source/API validator artifacts:\n" + "\n".join(missing))
    print(f"source/API artifact root is present: {root}")
    PY
    ```

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-source-api python3 - <<'PY'
import json
import os
from pathlib import Path

root_name = os.environ["ROOT_NAME"]
root = Path("validator/artifacts") / root_name
exit_code_path = root / "validator.exit-code"
if not exit_code_path.is_file():
    raise SystemExit(f"missing validator exit code: {exit_code_path}")
try:
    int(exit_code_path.read_text().strip())
except ValueError as exc:
    raise SystemExit(f"validator exit code is not numeric in {exit_code_path}") from exc

results_dir = root / "results/libpng"
summary_path = results_dir / "summary.json"
inventory_path = Path("validator-case-inventory.json")
if not summary_path.is_file():
    raise SystemExit(f"missing validator summary: {summary_path}")
if not inventory_path.is_file():
    raise SystemExit(f"missing validator inventory: {inventory_path}")

summary = json.loads(summary_path.read_text())
inventory = json.loads(inventory_path.read_text())
source_ids = inventory.get("source_case_ids", [])
usage_ids = inventory.get("usage_case_ids", [])
if not all(isinstance(case_id, str) for case_id in source_ids + usage_ids):
    raise SystemExit("inventory source_case_ids and usage_case_ids must contain only strings")
if len(set(source_ids)) != len(source_ids):
    raise SystemExit("inventory source_case_ids contains duplicates")
if len(set(usage_ids)) != len(usage_ids):
    raise SystemExit("inventory usage_case_ids contains duplicates")
if set(source_ids) & set(usage_ids):
    raise SystemExit("inventory source and usage case IDs overlap")
expected = set(source_ids) | set(usage_ids)
if inventory.get("cases") != len(expected):
    raise SystemExit(f"inventory cases={inventory.get('cases')} but expected ID count={len(expected)}")

required_summary = {
    "library": "libpng",
    "mode": "original",
    "cases": len(expected),
    "source_cases": len(source_ids),
    "usage_cases": len(usage_ids),
}
for key, expected_value in required_summary.items():
    if summary.get(key) != expected_value:
        raise SystemExit(f"summary {key}={summary.get(key)!r}, expected {expected_value!r}")
for key in ("passed", "failed", "casts"):
    if not isinstance(summary.get(key), int):
        raise SystemExit(f"summary {key} must be an integer")

result_paths = sorted(p for p in results_dir.glob("*.json") if p.name != "summary.json")
actual = {path.stem for path in result_paths}
missing = sorted(expected - actual)
extra = sorted(actual - expected)
if len(result_paths) != summary["cases"]:
    raise SystemExit(f"summary cases={summary['cases']} but found {len(result_paths)} per-case JSON files")
if missing or extra:
    raise SystemExit(f"validator result IDs differ from inventory; missing={missing} extra={extra}")

missing_artifacts = []
bad_overrides = []
for path in result_paths:
    result = json.loads(path.read_text())
    case_id = result.get("testcase_id", path.stem)
    if case_id != path.stem:
        raise SystemExit(f"testcase_id/path mismatch in {path}: {case_id}")
    if result.get("override_debs_installed") is not True:
        bad_overrides.append(case_id)
    for field in ("result_path", "log_path", "cast_path"):
        rel = result.get(field)
        if not isinstance(rel, str) or not (root / rel).is_file():
            missing_artifacts.append(f"{case_id}: missing {field} artifact {rel!r}")
if bad_overrides:
    raise SystemExit("override packages were not installed for: " + ", ".join(sorted(bad_overrides)))
if missing_artifacts:
    raise SystemExit("\n".join(missing_artifacts))

print(f"{root_name}: full libpng validator artifact root covers {len(expected)} inventory cases")
PY
```

```bash
python3 - <<'PY'
import subprocess

pathspecs = [
    "safe/debian/.debhelper",
    "safe/debian/tmp",
    "safe/debian/libpng-dev",
    "safe/debian/libpng-tools",
    "safe/debian/libpng16-16-udeb",
    "safe/debian/libpng16-16t64",
    "safe/debian/build-tools",
    "safe/debian/cargo-home",
    "safe/debian/upstream-source-root",
    "safe/debian/*.debhelper.log",
    "safe/debian/*.substvars",
    "safe/debian/files",
    "safe/debian/debhelper-build-stamp",
]

status = subprocess.run(
    ["git", "status", "--porcelain=v1", "--untracked-files=all", "--", *pathspecs],
    check=True,
    text=True,
    capture_output=True,
).stdout.splitlines()
staged_or_tracked_changes = [line for line in status if not line.startswith("?? ")]

tracked = subprocess.run(
    ["git", "ls-files", "--", *pathspecs],
    check=True,
    text=True,
    capture_output=True,
).stdout.splitlines()

if staged_or_tracked_changes or tracked:
    details = []
    if staged_or_tracked_changes:
        details.append("staged/tracked status entries:\n" + "\n".join(staged_or_tracked_changes))
    if tracked:
        details.append("tracked Debian build products:\n" + "\n".join(tracked))
    raise SystemExit("\n\n".join(details))

print("Debian build products are not staged or tracked")
PY
```

# Success Criteria

- Source/API failures from the baseline are either fixed with local regressions or documented as absent.
- `chunk-metadata-inspection` and `read-write-c-api-smoke` pass when present in the current inventory.
- Local ABI/read/write checks pass.
- Packages and override packages are rebuilt from current source.
- `validator/artifacts/libpng-safe-source-api/` is a fresh complete full-suite artifact root.
- `validator-report.md` records the phase result and any remaining failure classes.
- Validator suite files remain locally unmodified.
- Debian build products are not staged or tracked.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
