# Phase Name

Final clean validator run and report

# Implement Phase ID

`impl-final-clean-validation-and-report`

# Preexisting Inputs

Consume prior artifacts in place.

- All prior phase commits
- Latest validator checkout
- `validator/artifacts/libpng-safe-catch-all/` from Phase 6
- Final safe source tree and regression tests
- `validator-case-inventory.json`
- `validator-report.md`
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing override packages: `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Existing CVE/dependent inventory artifacts: `all_cves.json`, `relevant_cves.json`, `dependents.json`, and `safe/tests/cve-regressions/coverage.json`
- Existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`

# New Outputs

- Fresh final package artifacts
- Refreshed `validator-overrides/libpng/*.deb`
- Final validator artifacts under `validator/artifacts/libpng-safe-final/`
- Final `validator-report.md` with validator commit, checks, failures, fixes, package hashes, final summary, and exception status
- Final git commit before yielding

# File Changes

- `validator-report.md`: final authoritative report
- Root package artifacts if rebuilt outputs are tracked
- `validator-overrides/libpng/*.deb` if tracked or intentionally preserved for local review
- No validator suite edits

# Implementation Details

1. Start from a clean-enough worktree. Preserve unrelated user changes; do not reset or checkout over them.
2. Run the full local verification battery:

   ```bash
   cargo fmt --check --manifest-path safe/Cargo.toml
   cargo test --manifest-path safe/Cargo.toml
   safe/tools/check-core-smoke.sh
   safe/tools/check-read-core.sh
   safe/tools/check-read-transforms.sh
   safe/tools/run-cve-regressions.sh --mode all
   safe/tools/run-dependent-regressions.sh
   safe/tools/run-read-tests.sh
   safe/tools/run-write-tests.sh \
     pngstest-1.8 \
     pngstest-1.8-alpha \
     pngstest-large-stride \
     pngstest-linear \
     pngstest-linear-alpha \
     pngstest-none \
     pngstest-none-alpha \
     pngstest-sRGB \
     pngstest-sRGB-alpha
   safe/tools/run-upstream-tests.sh
   safe/tools/check-examples-and-tools.sh
   safe/tools/check-link-compat.sh
   safe/tools/check-exports.sh
   safe/tools/check-headers.sh
   safe/tools/check-install-surface.sh
   safe/tools/check-build-layout.sh
   ```

3. Rebuild final packages with `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`.
4. Run `safe/tools/check-package-artifacts.sh`.
5. Refresh `validator-overrides/libpng/` from final canonical `.deb` files.
6. Remove stale `validator/artifacts/libpng-safe-final/`, then run the full libpng validator from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-final" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-final/validator.exit-code`.
7. Parse final results. Summary must cover all current libpng cases in `validator-case-inventory.json`. Every expected case must have per-case JSON, log, and cast artifacts. Every result must pass with exit code 0 and `override_debs_installed: true` unless a documented validator bug exception exists.
8. Consume `proof_rejects_original_override` from `validator-case-inventory.json`; do not rediscover proof behavior in this phase.
   - If `proof_rejects_original_override` is `true`, do not modify validator proof tooling. Document this limitation in `validator-report.md` and use final result JSON, logs, casts, package hashes, and validator exit code as acceptance evidence.
   - If `proof_rejects_original_override` is `false`, run this exact proof verifier command from inside the current `validator/` checkout:

     ```bash
     cd validator
     source_cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["source_cases"])')
     usage_cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["usage_cases"])')
     cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["cases"])')
     python3 tools/verify_proof_artifacts.py \
       --config repositories.yml \
       --tests-root tests \
       --artifact-root artifacts/libpng-safe-final \
       --proof-output artifacts/libpng-safe-final/proof/libpng-original-override-proof.json \
       --mode original \
       --library libpng \
       --min-source-cases "$source_cases" \
       --min-usage-cases "$usage_cases" \
       --min-cases "$cases" \
       --require-casts
     ```
9. Rewrite `validator-report.md` with validator commit, all command lines, package hashes, final artifact root and exit code, final summary, failure classes, regressions added, fixes applied with commit IDs, and the exact `Validator Bug Exceptions:` line.
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
- Consume Phase 6 artifacts in place, especially `validator/artifacts/libpng-safe-catch-all/`, `validator-case-inventory.json`, `validator-report.md`, root package artifacts, `validator-overrides/libpng/*.deb`, CVE/dependent inventories, all local regression tests, and existing fixtures.
- Validator suite files are read-only except for clone or fast-forward updates made in Phase 1. `validator/repositories.yml`, `validator/tests/libpng/testcases.yml`, source and usage testcase scripts, `validator/tests/_shared/install_override_debs.sh`, `validator/tools/**`, and `validator/test.sh` must not be edited to hide failures.
- The final command battery in this phase is mandatory and must be run from `/home/yans/safelibs/pipeline/ports/port-libpng` before the final package build and validator run.
- Final acceptance requires `validator/artifacts/libpng-safe-final/results/libpng/summary.json`, all expected per-case JSON files, logs, casts, `override_debs_installed: true`, and `failed: 0` unless every remaining failure is listed as a proven validator bug exception.
- The full-suite artifact gate listed in this file's shared verifier command blocks must pass for every phase artifact root: `libpng-safe-initial`, `libpng-safe-source-api`, `libpng-safe-cli-source`, `libpng-safe-usage-netpbm`, `libpng-safe-usage-pngquant`, `libpng-safe-catch-all`, and `libpng-safe-final`.
- The Debian build-product commit gate must pass; no generated `safe/debian` build products may be staged or tracked.
- `git log --oneline` must show a linear sequence of implementation commits, one per implementation phase.

# Verification Phases

## `check-final-clean-validation-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-final-clean-validation-and-report`
- Purpose: independently verify the final command battery, package artifacts, validator summary, casts, and report.
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
  - `safe/tools/run-upstream-tests.sh`
  - `safe/tools/check-examples-and-tools.sh`
  - `safe/tools/check-link-compat.sh`
  - `safe/tools/check-exports.sh`
  - `safe/tools/check-headers.sh`
  - `safe/tools/check-install-surface.sh`
  - `safe/tools/check-build-layout.sh`
  - `safe/tools/check-package-artifacts.sh`
  - Run this final result check:

    ```bash
    python3 - <<'PY'
    import json
    import re
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-final")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    inventory_path = Path("validator-case-inventory.json")
    if not summary_path.is_file():
        raise SystemExit(f"missing final summary: {summary_path}")
    if not inventory_path.is_file():
        raise SystemExit(f"missing inventory: {inventory_path}")
    summary = json.loads(summary_path.read_text())
    inventory = json.loads(inventory_path.read_text())
    expected = set(inventory.get("source_case_ids", [])) | set(inventory.get("usage_case_ids", []))
    result_paths = sorted(p for p in results_dir.glob("*.json") if p.name != "summary.json")
    actual = {path.stem for path in result_paths}
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing or extra:
        raise SystemExit(f"final result IDs differ from inventory; missing={missing} extra={extra}")
    report = Path("validator-report.md").read_text()
    match = re.search(r"(?im)^Validator Bug Exceptions:\s*(.+)$", report)
    if not match:
        raise SystemExit("validator-report.md must contain 'Validator Bug Exceptions: none' or a comma-separated testcase list")
    raw = match.group(1).strip()
    exceptions = set() if raw.lower().rstrip(".") == "none" else {part.strip() for part in raw.split(",") if part.strip()}
    failures = []
    for path in result_paths:
        result = json.loads(path.read_text())
        case_id = result.get("testcase_id", path.stem)
        if result.get("override_debs_installed") is not True:
            raise SystemExit(f"{case_id}: override_debs_installed is not true")
        cast_rel = result.get("cast_path")
        if not isinstance(cast_rel, str) or not (root / cast_rel).is_file():
            raise SystemExit(f"{case_id}: missing cast_path artifact {cast_rel}")
        log_rel = result.get("log_path")
        if not isinstance(log_rel, str) or not (root / log_rel).is_file():
            raise SystemExit(f"{case_id}: missing log_path artifact {log_rel}")
        if result.get("status") != "passed" or result.get("exit_code") != 0:
            failures.append(case_id)
    if summary.get("cases") != len(expected):
        raise SystemExit(f"summary cases={summary.get('cases')} inventory cases={len(expected)}")
    if summary.get("casts") != len(expected):
        raise SystemExit(f"summary casts={summary.get('casts')} inventory cases={len(expected)}")
    if summary.get("failed") != len(failures):
        raise SystemExit(f"summary failed={summary.get('failed')} counted failures={len(failures)}")
    unexpected = sorted(set(failures) - exceptions)
    missing_exception_failures = sorted(exceptions - set(failures))
    if unexpected:
        raise SystemExit("unexpected final failures: " + ", ".join(unexpected))
    if missing_exception_failures:
        raise SystemExit("reported validator bug exceptions are not failing in final results: " + ", ".join(missing_exception_failures))
    if not exceptions and summary.get("failed") != 0:
        raise SystemExit(f"final failed count must be 0 without exceptions: {summary}")
    print(f"final validator results match inventory: cases={len(expected)} failures={len(failures)} exceptions={', '.join(sorted(exceptions)) or 'none'}")
    PY
    ```

  - If `validator-case-inventory.json` has `proof_rejects_original_override: false`, run the top-level "Final proof verifier command" below; if it has `proof_rejects_original_override: true`, verify `validator-report.md` documents that proof tooling limitation.
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-final`.
  - Inline and run the Debian build-product commit gate below.

## `check-final-clean-validation-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-final-clean-validation-and-report`
- Purpose: review workflow compliance, artifact flow, and report completeness.
- Commands:
  - `git status --short --branch`
  - `git log --oneline -n 12`
  - `test -z "$(git -C validator status --short -- tests tools repositories.yml test.sh)"`
  - `git -C validator diff --exit-code -- tests tools repositories.yml test.sh`
  - `rg -n "Validator commit|Checks Executed|Failures Found|Fixes Applied|Final Validator Results|Validator Bug Exceptions|Package Artifacts" validator-report.md`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-final`.
  - Inline and run the Debian build-product commit gate below.
  - Run the top-level "Final all-root artifact gate" command block below to satisfy final acceptance for every phase artifact root.
  - Run this inventory/result review:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    inventory = json.loads(Path("validator-case-inventory.json").read_text())
    expected = set(inventory.get("source_case_ids", [])) | set(inventory.get("usage_case_ids", []))
    results_dir = Path("validator/artifacts/libpng-safe-final/results/libpng")
    actual = {path.stem for path in results_dir.glob("*.json") if path.name != "summary.json"}
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing or extra:
        raise SystemExit(f"inventory/result mismatch; missing={missing} extra={extra}")
    summary = json.loads((results_dir / "summary.json").read_text())
    if summary.get("cases") != len(expected):
        raise SystemExit(f"summary case count mismatch: summary={summary.get('cases')} inventory={len(expected)}")
    print(f"inventory/result IDs match: {len(expected)} cases")
    PY
    ```

## Final proof verifier command

Run this from `/home/yans/safelibs/pipeline/ports/port-libpng` when `validator-case-inventory.json` has `proof_rejects_original_override: false`. When that field is `true`, do not run this command; instead verify `validator-report.md` documents the proof tooling limitation.

```bash
cd validator
source_cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["source_cases"])')
usage_cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["usage_cases"])')
cases=$(python3 -c 'import json; print(json.load(open("../validator-case-inventory.json"))["cases"])')
python3 tools/verify_proof_artifacts.py \
  --config repositories.yml \
  --tests-root tests \
  --artifact-root artifacts/libpng-safe-final \
  --proof-output artifacts/libpng-safe-final/proof/libpng-original-override-proof.json \
  --mode original \
  --library libpng \
  --min-source-cases "$source_cases" \
  --min-usage-cases "$usage_cases" \
  --min-cases "$cases" \
  --require-casts
```

## Final all-root artifact gate

Run this command during `check-final-clean-validation-senior-tester`:

```bash
for ROOT_NAME in \
  libpng-safe-initial \
  libpng-safe-source-api \
  libpng-safe-cli-source \
  libpng-safe-usage-netpbm \
  libpng-safe-usage-pngquant \
  libpng-safe-catch-all \
  libpng-safe-final
do
  ROOT_NAME="$ROOT_NAME" python3 - <<'PY'
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
done
```

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-final python3 - <<'PY'
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

- Full local command battery passes.
- Final package artifacts are rebuilt and checked.
- `validator/artifacts/libpng-safe-final/` contains a fresh complete full-suite run with exit code, per-case JSON, logs, and casts.
- Final results cover every current libpng case in `validator-case-inventory.json`.
- Final summary has `failed: 0` unless all remaining failures are documented validator bug exceptions.
- `validator-report.md` contains validator commit, commands, package hashes, failures found, regression tests, fixes applied, final summary, and validator bug exception status.
- If `proof_rejects_original_override` is false, the proof verifier command succeeds and writes `validator/artifacts/libpng-safe-final/proof/libpng-original-override-proof.json`; if it is true, `validator-report.md` documents that proof limitation and acceptance evidence.
- All phase artifact gates pass.
- Debian build products are not staged or tracked.
- Git history shows one implementation commit per implementation phase.
- Validator suite files remain locally unmodified.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
