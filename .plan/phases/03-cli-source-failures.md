# Phase Name

Fix CLI and source fixture validator failures

# Implement Phase ID

`impl-cli-source-validator-failures`

# Preexisting Inputs

Consume prior artifacts in place.

- `validator/artifacts/libpng-safe-source-api/` from Phase 2
- `validator-case-inventory.json`
- `validator-report.md`
- Current `validator/` checkout
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing override packages under `validator-overrides/libpng/`
- Validator source scripts when present:
  - `validator/tests/libpng/tests/cases/source/malformed-png-rejection.sh`
  - `validator/tests/libpng/tests/cases/source/palette-fixture-handling.sh`
  - `validator/tests/libpng/tests/cases/source/pngfix-fixture-handling.sh`
- Safe packaged tool build inputs:
  - `safe/contrib/tools/pngfix.c`
  - `safe/contrib/tools/png-fix-itxt.c`
  - `safe/debian/rules`
  - `safe/tests/upstream/pngfix.sh`
  - `safe/tools/check-examples-and-tools.sh`
- Existing upstream/test fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, and `safe/contrib/pngsuite`

# New Outputs

- Minimal local regression test for any failing CLI/source fixture behavior
- Safe library, tool, or packaging fix as needed
- Rebuilt package artifacts and refreshed override packages
- Full validator rerun artifacts under `validator/artifacts/libpng-safe-cli-source/`, produced by this phase even when no CLI/source failures exist
- Updated `validator-report.md`
- A git commit before yielding

# File Changes

- Candidate test updates:
  - `safe/tests/upstream/pngfix.sh`
  - `safe/tools/check-examples-and-tools.sh`
  - `safe/tests/core-smoke/*.c`
- Candidate implementation fixes:
  - `safe/src/read.rs`
  - `safe/src/read_util.rs`
  - `safe/src/chunks.rs`
  - `safe/src/write_runtime.rs`
  - `safe/src/set.rs`
  - `safe/src/get.rs`
  - `safe/contrib/tools/pngfix.c`
  - `safe/debian/rules`
- `validator-report.md`
- Package artifacts and `validator-overrides/libpng/` as needed after rebuild
- Do not edit validator testcase scripts to make failures pass.

# Implementation Details

1. Inspect `validator/artifacts/libpng-safe-source-api/` for `malformed-png-rejection`, `palette-fixture-handling`, and `pngfix-fixture-handling`.
2. If all three cases passed, document that no CLI/source fix was needed, rebuild packages, refresh overrides, rerun the full validator to `validator/artifacts/libpng-safe-cli-source/`, verify, clean Debian build products, run the commit gate, commit, and continue.
3. For `malformed-png-rejection`, reproduce locally with a non-PNG byte stream. Expected behavior: `pngfix --out=<path> bad.png` exits nonzero and does not produce a valid output file.
4. For `palette-fixture-handling`, reproduce with `basn3p08.png` and assert output exists after `pngfix --out`.
5. For `pngfix-fixture-handling`, reproduce with `basn2c08.png`, assert `pngfix` exits zero, and assert the fixed file is readable or recognized as PNG.
6. Fix root cause in the safe library first. Touch `safe/contrib/tools/pngfix.c` or packaging only when the failure is clearly tool packaging or invocation behavior.
7. Preserve existing upstream fixtures. Do not copy or regenerate PNGSuite files unless a new tiny generated fixture is necessary and committed under an existing test fixture directory.
8. Rebuild packages, refresh `validator-overrides/libpng/`, remove stale `validator/artifacts/libpng-safe-cli-source/`, and run the full libpng validator from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-cli-source" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-cli-source/validator.exit-code`.
9. Update `validator-report.md` with commands, local regression test paths, package hashes if changed, and remaining failures.
10. Remove generated Debian build directories and files, run the Debian build-product commit gate below, and commit before yielding.

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
- Consume Phase 2 artifacts in place, especially `validator/artifacts/libpng-safe-source-api/`, `validator-case-inventory.json`, `validator-report.md`, root package artifacts, `validator-overrides/libpng/*.deb`, and existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`.
- The CLI/source validator scripts are reproduction inputs only: `validator/tests/libpng/tests/cases/source/malformed-png-rejection.sh`, `validator/tests/libpng/tests/cases/source/palette-fixture-handling.sh`, and `validator/tests/libpng/tests/cases/source/pngfix-fixture-handling.sh` must not be edited.
- Validator suite files are read-only except for clone or fast-forward updates made in Phase 1. Review with `git -C validator status --short` and `git -C validator diff -- tests tools repositories.yml test.sh`.
- Keep fixes in safe library, packaged tool, or packaging files. Do not change validator tests, manifests, runner logic, or shared install scripts to make failures pass.
- Treat `safe/debian/rules`, `safe/debian/control`, public headers, ABI baselines, and install-layout files as critical; touch them only when the failure is clearly packaging, tool shipment, ABI, or install-surface behavior and document that reason in `validator-report.md`.
- Every implementation phase must rebuild packages, refresh `validator-overrides/libpng/`, run a fresh full-suite validator root, write `validator.exit-code`, update `validator-report.md`, run the Debian build-product cleanup and commit gate, and commit before yielding.

# Verification Phases

## `check-cli-source-validator-failures-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-cli-source-validator-failures`
- Purpose: confirm `pngfix`, malformed PNG rejection, and palette fixture cases pass with local regressions.
- Commands:
  - `cargo fmt --check --manifest-path safe/Cargo.toml`
  - `safe/tools/check-examples-and-tools.sh`
  - `safe/tools/run-upstream-tests.sh`
  - `safe/tests/upstream/pngfix.sh`
  - `safe/tools/check-core-smoke.sh`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Run this CLI/source result check:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    inventory = json.loads(Path("validator-case-inventory.json").read_text())
    expected = [
        case_id
        for case_id in ("malformed-png-rejection", "palette-fixture-handling", "pngfix-fixture-handling")
        if case_id in set(inventory.get("source_case_ids", []))
    ]
    root = Path("validator/artifacts/libpng-safe-cli-source")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"missing CLI/source summary: {summary_path}")
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
    print(f"CLI/source cases passed in {root}: {', '.join(expected) if expected else 'none in inventory'}")
    PY
    ```

  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-cli-source`.
  - Inline and run the Debian build-product commit gate below.

## `check-cli-source-validator-failures-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-cli-source-validator-failures`
- Purpose: confirm fixes are in `safe/` packaging, library, or tool code, not validator tests.
- Commands:
  - `git diff HEAD~1..HEAD -- safe validator-report.md`
  - `test -z "$(git -C validator status --short -- tests/libpng tests/_shared tools test.sh)"`
  - `git -C validator diff --exit-code -- tests/libpng tests/_shared tools test.sh`
  - `rg -n "malformed-png-rejection|palette-fixture-handling|pngfix-fixture-handling|CLI/source" validator-report.md`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-cli-source`.
  - Inline and run the Debian build-product commit gate below.

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-cli-source python3 - <<'PY'
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

- CLI/source fixture failures are either fixed with local regressions or documented as absent.
- `malformed-png-rejection`, `palette-fixture-handling`, and `pngfix-fixture-handling` pass when present in the current inventory.
- Tool and upstream smoke checks pass.
- Packages and override packages are rebuilt from current source.
- `validator/artifacts/libpng-safe-cli-source/` is a fresh complete full-suite artifact root.
- `validator-report.md` records the phase result and remaining failures.
- Validator suite files remain locally unmodified.
- Debian build products are not staged or tracked.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
