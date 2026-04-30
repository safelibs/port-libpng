# Phase Name

Fix all remaining validator failures

# Implement Phase ID

`impl-catch-all-validator-failures`

# Preexisting Inputs

Consume prior artifacts in place.

- `validator/artifacts/libpng-safe-usage-pngquant/` from Phase 5
- All local safe regression tests from previous phases
- All validator logs from previous phases
- `validator-report.md`
- `validator-case-inventory.json`
- Current `validator/` checkout
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing override packages under `validator-overrides/libpng/`
- Existing CVE/dependent inventory artifacts: `all_cves.json`, `relevant_cves.json`, `dependents.json`, and `safe/tests/cve-regressions/coverage.json`
- Existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`

# New Outputs

- Additional local regressions for any remaining failure classes
- Safe fixes for residual compatibility issues
- Documented validator bug exception in `validator-report.md` only if proven
- Full validator rerun artifacts under `validator/artifacts/libpng-safe-catch-all/`, produced by this phase even when no residual failures remain
- Updated package artifacts and overrides
- A git commit before yielding

# File Changes

- Any `safe/src/**` file necessary for a proven residual libpng-safe issue
- Any appropriate `safe/tests/**` regression file
- `safe/tools/**` only if a local test runner must include a newly added regression category
- `validator-report.md`
- `validator-case-inventory.json` only if validator commit, inventory, grouping, override-support, or proof-behavior fields changed
- Package artifacts and `validator-overrides/libpng/` as needed after rebuild
- Do not edit validator tests to skip failures.

# Implementation Details

1. Parse `validator/artifacts/libpng-safe-usage-pngquant/results/libpng/*.json` and list all cases still failing.
2. If none are failing, document that no catch-all fix was needed, set `Validator Bug Exceptions: none`, rebuild packages, refresh overrides, rerun the full validator to `validator/artifacts/libpng-safe-catch-all/`, verify, clean Debian build products, run the commit gate, commit, and continue.
3. For each remaining failure, add a minimal local regression under `safe/`, fix the underlying safe implementation, run the local reproducer, rebuild packages, refresh overrides, and rerun validator.
4. If a failure appears to be a validator bug, prove it by comparing validator expectations, original Ubuntu package behavior if needed, upstream libpng documented behavior, and safe behavior through a direct local reproducer.
5. If a validator bug is proven, do not modify validator tests. Keep the full unfiltered failing artifact root, document testcase ID, log path, expected behavior, observed original behavior, why the validator is wrong, and how only that check is excluded from final acceptance.
6. Set `Validator Bug Exceptions: <testcase-id>` or a comma-separated list only for proven validator bugs. Otherwise set `Validator Bug Exceptions: none`.
7. Run the broad local battery and full libpng validator to `validator/artifacts/libpng-safe-catch-all/` from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-catch-all" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-catch-all/validator.exit-code`.
8. Update `validator-report.md` with all residual fixes or exceptions.
9. Remove generated Debian build directories and files, run the Debian build-product commit gate below, and commit before yielding.

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
- Consume Phase 5 artifacts in place, especially `validator/artifacts/libpng-safe-usage-pngquant/`, `validator-case-inventory.json`, `validator-report.md`, root package artifacts, `validator-overrides/libpng/*.deb`, prior validator logs, local regression tests, CVE/dependent inventories, and existing fixtures.
- Validator suite files are read-only except for clone or fast-forward updates made in Phase 1. Review with `git -C validator status --short` and `git -C validator diff -- tests tools repositories.yml test.sh`.
- If a validator bug is proven, do not modify validator tests. Keep the full unfiltered failing artifact root, document the testcase ID, log path, expected behavior, observed original behavior, and justification in `validator-report.md`, then list only those IDs on the `Validator Bug Exceptions:` line.
- If no validator bug exception is proven, `validator-report.md` must contain the exact line `Validator Bug Exceptions: none`.
- Treat public headers, ABI baselines, `safe/build.rs`, `safe/debian/rules`, `safe/debian/control`, and install-layout files as critical; change them only for a documented compatibility or install-surface issue.
- Every implementation phase must rebuild packages, refresh `validator-overrides/libpng/`, run a fresh full-suite validator root, write `validator.exit-code`, update `validator-report.md`, run the Debian build-product cleanup and commit gate, and commit before yielding.

# Verification Phases

## `check-catch-all-validator-failures-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-catch-all-validator-failures`
- Purpose: confirm no known class of validator failure remains and local test coverage is broad.
- Commands:
  - `cargo fmt --check --manifest-path safe/Cargo.toml`
  - `cargo test --manifest-path safe/Cargo.toml`
  - `safe/tools/check-core-smoke.sh`
  - `safe/tools/check-read-core.sh`
  - `safe/tools/check-read-transforms.sh`
  - `safe/tools/run-cve-regressions.sh --mode all`
  - `safe/tools/run-dependent-regressions.sh`
  - `safe/tools/run-read-tests.sh`
  - `safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-large-stride pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Run this catch-all result check:

    ```bash
    python3 - <<'PY'
    import json
    import re
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-catch-all")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"missing catch-all summary: {summary_path}")
    summary = json.loads(summary_path.read_text())
    report = Path("validator-report.md").read_text()
    match = re.search(r"(?im)^Validator Bug Exceptions:\s*(.+)$", report)
    if not match:
        raise SystemExit("validator-report.md must contain 'Validator Bug Exceptions: none' or a comma-separated testcase list")
    raw = match.group(1).strip()
    exceptions = set() if raw.lower().rstrip(".") == "none" else {part.strip() for part in raw.split(",") if part.strip()}
    result_paths = sorted(p for p in results_dir.glob("*.json") if p.name != "summary.json")
    failed = []
    for path in result_paths:
        result = json.loads(path.read_text())
        case_id = result.get("testcase_id", path.stem)
        if result.get("status") != "passed" or result.get("exit_code") != 0:
            failed.append(case_id)
        if result.get("override_debs_installed") is not True:
            raise SystemExit(f"{case_id}: override_debs_installed is not true")
    if summary.get("failed") != len(failed):
        raise SystemExit(f"summary failed={summary.get('failed')} but counted {len(failed)} failed result files")
    unexpected = sorted(set(failed) - exceptions)
    missing_exception_failures = sorted(exceptions - set(failed))
    if unexpected:
        raise SystemExit("unexpected catch-all failures: " + ", ".join(unexpected))
    if missing_exception_failures:
        raise SystemExit("reported validator bug exceptions are not failing in catch-all results: " + ", ".join(missing_exception_failures))
    if not exceptions and summary.get("failed") != 0:
        raise SystemExit(f"catch-all failed count must be 0 without exceptions: {summary}")
    print(f"catch-all summary accepted: failed={summary.get('failed')} exceptions={', '.join(sorted(exceptions)) or 'none'}")
    PY
    ```

  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-catch-all`.
  - Inline and run the Debian build-product commit gate below.

## `check-catch-all-validator-failures-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-catch-all-validator-failures`
- Purpose: review remaining failures, exception reasoning, and regression coverage quality.
- Commands:
  - `git diff HEAD~1..HEAD -- safe validator-report.md validator-case-inventory.json`
  - `rg -n "remaining|catch-all|validator bug|exception|skip" validator-report.md`
  - `test -z "$(git -C validator status --short -- tests tools repositories.yml test.sh)"`
  - `git -C validator diff --exit-code -- tests tools repositories.yml test.sh`
  - `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`
  - `safe/tools/check-package-artifacts.sh`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-catch-all`.
  - Inline and run the Debian build-product commit gate below.
  - Run this exception evidence check:

    ```bash
    python3 - <<'PY'
    import json
    import re
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-catch-all")
    results_dir = root / "results/libpng"
    report = Path("validator-report.md").read_text()
    match = re.search(r"(?im)^Validator Bug Exceptions:\s*(.+)$", report)
    if not match:
        raise SystemExit("validator-report.md is missing the Validator Bug Exceptions line")
    raw = match.group(1).strip()
    exceptions = set() if raw.lower().rstrip(".") == "none" else {part.strip() for part in raw.split(",") if part.strip()}
    if not exceptions:
        print("no validator bug exceptions documented")
        raise SystemExit(0)
    for case_id in sorted(exceptions):
        path = results_dir / f"{case_id}.json"
        if not path.is_file():
            raise SystemExit(f"missing result for exception {case_id}")
        result = json.loads(path.read_text())
        if result.get("status") == "passed" and result.get("exit_code") == 0:
            raise SystemExit(f"exception {case_id} passed and should be removed from the report")
        log_rel = result.get("log_path")
        log_path = root / log_rel if isinstance(log_rel, str) else None
        if log_path is None or not log_path.is_file():
            raise SystemExit(f"missing unfiltered log for exception {case_id}: {log_rel}")
        print(f"exception {case_id}: result={path} log={log_path}")
    PY
    ```

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-catch-all python3 - <<'PY'
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

- All residual validator failures are either fixed with local regressions or documented as proven validator bug exceptions.
- `Validator Bug Exceptions: none` is present unless a precise comma-separated exception list is justified.
- Broad local test battery passes.
- Packages and override packages are rebuilt from current source.
- `validator/artifacts/libpng-safe-catch-all/` is a fresh complete full-suite artifact root.
- `validator-report.md` records residual fixes or exception evidence.
- Validator suite files remain locally unmodified.
- Debian build products are not staged or tracked.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
