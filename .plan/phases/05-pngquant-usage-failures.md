# Phase Name

Fix pngquant dependent usage validator failures

# Implement Phase ID

`impl-pngquant-usage-validator-failures`

# Preexisting Inputs

Consume prior artifacts in place.

- `validator/artifacts/libpng-safe-usage-netpbm/` from Phase 4
- `validator-case-inventory.json`
- `validator-report.md`
- Current `validator/` checkout
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing override packages under `validator-overrides/libpng/`
- Validator pngquant usage scripts currently present in the validator checkout:
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-cant-open-input-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-eight-png-generated.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-eight-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-four-png-generated.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-four-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-sixteen-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-three-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-colors-two-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-compress-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-ext-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-floyd-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-floyd-zero-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-iebug-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-map-palette-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-nofs-png-generated.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-nofs-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-posterize-one-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-posterize-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-posterize-two-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-high-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-low-png-generated.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-low-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-mid-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-min-only-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-quality-range-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-skip-if-larger-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-eleven-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-five-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-one-png-generated.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-one-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-speed-three-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-strip-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-transbug-png.sh`
  - `validator/tests/libpng/tests/cases/usage/usage-pngquant-verbose-png.sh`
- `validator/tests/libpng/tests/fixtures/dependents.json`, which lists `pngquant`
- Existing dependent regression `safe/tests/dependents/write_packing_indices.c`
- Existing read/write transform and write wrapper tests
- Existing upstream/test fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, and `safe/contrib/pngsuite`

# New Outputs

- Minimal local regression tests under `safe/tests/dependents/` or `safe/tests/read-transforms/`
- Safe code fixes for pngquant compatibility
- Rebuilt packages and refreshed overrides
- Full validator rerun artifacts under `validator/artifacts/libpng-safe-usage-pngquant/`, produced by this phase even when no pngquant failures exist
- Updated `validator-report.md`
- A git commit before yielding

# File Changes

- Candidate test files:
  - `safe/tests/dependents/validator_pngquant_<behavior>.c`
  - `safe/tests/dependents/write_packing_indices.c`
  - `safe/tests/read-transforms/read_png_driver.c`
  - `safe/tests/read-transforms/simplified_read_driver.c`
- Candidate implementation files:
  - `safe/src/write_runtime.rs`
  - `safe/src/write_transform.rs`
  - `safe/src/read_transform.rs`
  - `safe/src/simplified_runtime.rs`
  - `safe/src/chunks.rs`
  - `safe/src/get.rs`
  - `safe/src/set.rs`
  - `safe/src/state.rs`
- `validator-report.md`
- Package artifacts and `validator-overrides/libpng/` as needed after rebuild
- Do not edit validator testcase scripts to make failures pass.

# Implementation Details

1. Identify failing `usage-pngquant-*` cases from `validator/artifacts/libpng-safe-usage-netpbm/results/libpng/*.json`.
2. If no `usage-pngquant-*` cases failed, document that no pngquant fix was needed, rebuild packages, refresh overrides, rerun the full validator to `validator/artifacts/libpng-safe-usage-pngquant/`, verify, clean Debian build products, run the commit gate, commit, and continue.
3. Inspect the corresponding shell scripts and logs. Group failures by behavior rather than option name.
4. Typical pngquant-exposed behaviors include palette handling, alpha/transparency, tRNS, packed palette row input/output, skip-if-larger behavior, output metadata, and read/write round trips.
5. Add the smallest local C regression for each distinct behavior. Use `safe/tests/dependents/write_packing_indices.c` as the pattern for write-side packing failures.
6. In `safe/src/write_runtime.rs`, inspect `write_layout_from_info`, `write_transform_spec`, packed pixel helpers, row write APIs, `write_end`, and PLTE/tRNS/text/chunk emission helpers.
7. On the read side, ensure palette and alpha metadata returned through getters remains stable and consistent after transforms.
8. Do not fix dependent-client behavior by changing validator scripts or weakening validation.
9. Run local checks, rebuild packages, refresh `validator-overrides/libpng/`, remove stale `validator/artifacts/libpng-safe-usage-pngquant/`, and run the full libpng validator from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-usage-pngquant" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-usage-pngquant/validator.exit-code`.
10. Update `validator-report.md` with the pngquant case list, local regressions, source changes, commands, and remaining failures.
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
- Consume Phase 4 artifacts in place, especially `validator/artifacts/libpng-safe-usage-netpbm/`, `validator-case-inventory.json`, `validator-report.md`, root package artifacts, `validator-overrides/libpng/*.deb`, and existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`.
- pngquant validator usage scripts under `validator/tests/libpng/tests/cases/usage/usage-pngquant-*.sh` and `validator/tests/libpng/tests/fixtures/dependents.json` are reproduction inputs only and must not be edited.
- Validator suite files are read-only except for clone or fast-forward updates made in Phase 1. Review with `git -C validator status --short` and `git -C validator diff -- tests tools repositories.yml test.sh`.
- Keep fixes in the safe libpng API surface, write/read transform runtime, palette/alpha metadata handling, simplified runtime, state, or local regression harnesses. Do not weaken validator usage scripts or dependent setup.
- Treat public headers, ABI baselines, `safe/build.rs`, `safe/debian/rules`, and install-layout files as critical; change them only for a documented compatibility or install-surface issue.
- Every implementation phase must rebuild packages, refresh `validator-overrides/libpng/`, run a fresh full-suite validator root, write `validator.exit-code`, update `validator-report.md`, run the Debian build-product cleanup and commit gate, and commit before yielding.

# Verification Phases

## `check-pngquant-usage-validator-failures-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-pngquant-usage-validator-failures`
- Purpose: confirm pngquant-dependent failures have local regressions and all `usage-pngquant-*` cases pass.
- Commands:
  - `cargo fmt --check --manifest-path safe/Cargo.toml`
  - `cargo test --manifest-path safe/Cargo.toml`
  - `safe/tools/run-dependent-regressions.sh`
  - `safe/tools/check-read-transforms.sh`
  - `safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha pngstest-large-stride`
  - `safe/tools/check-examples-and-tools.sh`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Run this pngquant result check:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-usage-pngquant")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"missing pngquant summary: {summary_path}")
    result_paths = sorted(results_dir.glob("usage-pngquant-*.json"))
    if not result_paths:
        raise SystemExit(f"no usage-pngquant result JSON files under {results_dir}")
    failures = []
    for path in result_paths:
        result = json.loads(path.read_text())
        if result.get("status") != "passed" or result.get("exit_code") != 0:
            failures.append(f"{path.stem}: status={result.get('status')} exit_code={result.get('exit_code')} log={result.get('log_path')}")
        if result.get("override_debs_installed") is not True:
            failures.append(f"{path.stem}: override_debs_installed is not true")
    if failures:
        raise SystemExit("\n".join(failures))
    print(f"pngquant usage cases passed in {root}: {len(result_paths)}")
    PY
    ```

  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-usage-pngquant`.
  - Inline and run the Debian build-product commit gate below.

## `check-pngquant-usage-validator-failures-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-pngquant-usage-validator-failures`
- Purpose: review palette, alpha, packing, and write semantics for general libpng compatibility.
- Commands:
  - `git diff HEAD~1..HEAD -- safe/src safe/tests safe/tools validator-report.md`
  - `test -z "$(git -C validator status --short -- tests tools repositories.yml test.sh)"`
  - `git -C validator diff --exit-code -- tests tools repositories.yml test.sh`
  - `rg -n "usage-pngquant|pngquant|palette|packing|alpha" validator-report.md`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-usage-pngquant`.
  - Inline and run the Debian build-product commit gate below.
  - Run this before/after review:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    before_root = Path("validator/artifacts/libpng-safe-usage-netpbm")
    after_root = Path("validator/artifacts/libpng-safe-usage-pngquant")
    before_results = before_root / "results/libpng"
    after_results = after_root / "results/libpng"
    if not before_results.is_dir() or not after_results.is_dir():
        raise SystemExit("missing before or after pngquant result directory")
    before_failures = []
    for path in sorted(before_results.glob("usage-pngquant-*.json")):
        result = json.loads(path.read_text())
        if result.get("status") != "passed":
            before_failures.append(path.stem)
    if not before_failures:
        print("no failing-before pngquant cases were present in the Phase 4 artifact root")
        raise SystemExit(0)
    for case_id in before_failures[:5]:
        after_path = after_results / f"{case_id}.json"
        if not after_path.is_file():
            raise SystemExit(f"missing after result for {case_id}")
        after = json.loads(after_path.read_text())
        if after.get("status") != "passed":
            raise SystemExit(f"{case_id} did not pass after pngquant phase: {after.get('status')} log={after.get('log_path')}")
        before_log = before_root / f"logs/libpng/{case_id}.log"
        after_log = after_root / f"logs/libpng/{case_id}.log"
        if not before_log.is_file() or not after_log.is_file():
            raise SystemExit(f"missing before/after log pair for {case_id}")
        print(f"{case_id}: before={before_log} after={after_log}")
    PY
    ```

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-usage-pngquant python3 - <<'PY'
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

- pngquant usage failures are either fixed with local regressions or documented as absent.
- All `usage-pngquant-*` result JSON files in the phase artifact root pass.
- Local dependent/read/write/tool checks pass.
- Packages and override packages are rebuilt from current source.
- `validator/artifacts/libpng-safe-usage-pngquant/` is a fresh complete full-suite artifact root.
- `validator-report.md` records the phase result and remaining failures.
- Validator suite files remain locally unmodified.
- Debian build products are not staged or tracked.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
