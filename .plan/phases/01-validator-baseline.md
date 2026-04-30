# Phase Name

Baseline validator setup and initial libpng-safe run

# Implement Phase ID

`impl-validator-baseline`

# Preexisting Inputs

Consume existing local artifacts in place. If an artifact already exists, update it only as directed here; do not rediscover or regenerate it from scratch.

- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/original/`
- `validator/`, if already present as the ignored nested validator checkout
- `validator-report.md`
- `validator-case-inventory.json`
- `validator-overrides/libpng/`
- Existing root package artifacts: `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`, and `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- `all_cves.json`
- `relevant_cves.json`
- `dependents.json`
- `safe/tests/cve-regressions/coverage.json`
- Existing fixtures under `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`

# New Outputs

- Updated or cloned `validator/` checkout
- Fresh local override packages under `validator-overrides/libpng/`
- Initial validator artifacts under `validator/artifacts/libpng-safe-initial/`
- `validator/artifacts/libpng-safe-initial/validator.exit-code`
- Updated `validator-case-inventory.json` for the post-update validator commit and current libpng case inventory
- Rewritten initial sections of `validator-report.md`
- A git commit, for example `validator: capture initial libpng-safe validator run`

# File Changes

- `validator-report.md`: rewrite setup, validator commit, package hashes, initial artifact root, initial summary, and initial failure classification.
- `validator-case-inventory.json`: update when validator commit, case counts or IDs, source/usage grouping, original-mode override support, or proof behavior differs.
- `validator-overrides/libpng/libpng16-16t64_*.deb`, `validator-overrides/libpng/libpng-dev_*.deb`, and `validator-overrides/libpng/libpng-tools_*.deb`: refresh from rebuilt packages.
- Root package artifacts may refresh when the Debian package rebuild changes tracked outputs.
- Do not edit `validator/tests/**`, `validator/tools/**`, `validator/repositories.yml`, or `validator/test.sh`.

# Implementation Details

1. From `/home/yans/safelibs/pipeline/ports/port-libpng`, update the validator checkout. If `validator/.git` exists, run `git -C validator pull --ff-only`; otherwise clone `https://github.com/safelibs/validator` into `validator/`.
2. If the validator checkout has local uncommitted changes before pulling, stop, document the blocker in `validator-report.md`, and do not edit validator files.
3. Record `git -C validator rev-parse HEAD` after clone or pull. Use that exact commit in `validator-case-inventory.json` and `validator-report.md`.
4. Read `validator/README.md` and use its local override package contract. Do not use the official release-fetch flow for this local source tree unless needed for comparison.
5. Run validator tooling checks: `cd validator && make unit` and `cd validator && make check-testcases`.
6. Build libpng-safe packages from source with `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`, then run `safe/tools/check-package-artifacts.sh`.
7. Refresh `validator-overrides/libpng/` by removing only stale files in that leaf and copying the canonical `libpng16-16t64`, `libpng-dev`, and `libpng-tools` `.deb` artifacts from the repo root. Do not copy `.ddeb`, `.udeb`, source tarballs, `.changes`, or `.buildinfo` files into the override leaf.
8. Recompute `validator-case-inventory.json` from the current validator libpng testcase manifests and runner behavior. Preserve existing grouping fields only when still true. New or renamed source cases belong in catch-all unless their behavior clearly belongs earlier.
9. Remove stale `validator/artifacts/libpng-safe-initial/`, then run the full libpng validator from inside `validator/`:

   ```bash
   bash test.sh \
     --config repositories.yml \
     --tests-root tests \
     --artifact-root "$PWD/artifacts/libpng-safe-initial" \
     --mode original \
     --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
     --library libpng \
     --record-casts
   ```

   Write the numeric exit status to `validator/artifacts/libpng-safe-initial/validator.exit-code` even when nonzero.
10. Parse `validator/artifacts/libpng-safe-initial/results/libpng/summary.json` and per-case JSON files. Classify failures with this explicit mapping:
    - Source/API: `chunk-metadata-inspection`, `read-write-c-api-smoke`.
    - CLI/source fixtures: `malformed-png-rejection`, `palette-fixture-handling`, `pngfix-fixture-handling`.
    - Netpbm usage: testcase IDs beginning `usage-netpbm-`.
    - pngquant usage: testcase IDs beginning `usage-pngquant-`.
    - Other/catch-all: every failing testcase ID not covered by the named source/API, named CLI/source, Netpbm, or pngquant groups.
11. Rewrite `validator-report.md` with the validator commit, commands executed, package artifacts and SHA-256 hashes, initial summary, failure classification, and artifact paths.
12. Remove generated Debian build directories and files, run the Debian build-product commit gate listed below, and commit all tracked changes before yielding. Do not commit ignored validator artifacts, ignored override packages, or Debian build products.

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
- Preserve the consume-existing-artifacts contract: update `validator-report.md`, `validator-case-inventory.json`, root package artifacts, and `validator-overrides/libpng/*.deb` in place instead of recollecting or regenerating unrelated artifacts.
- `validator/` is an ignored nested git checkout. It may be cloned or fast-forward updated, but `validator/repositories.yml`, `validator/tests/libpng/testcases.yml`, `validator/tests/libpng/tests/cases/source/*.sh`, `validator/tests/libpng/tests/cases/usage/*.sh`, `validator/tests/_shared/install_override_debs.sh`, `validator/tools/**`, and `validator/test.sh` must not be edited to hide failures.
- Use `validator/README.md` as the source for the local override contract. The validator consumes packages from `validator-overrides/libpng/*.deb`, not the Rust crate path.
- Consume existing CVE/dependent inventories and fixtures: `all_cves.json`, `relevant_cves.json`, `dependents.json`, `safe/tests/cve-regressions/coverage.json`, `original/contrib/pngsuite`, `original/contrib/testpngs`, `safe/contrib/pngsuite`, and `validator/tests/libpng/tests/fixtures/samples`.
- Safe ABI, header, and install-surface files are critical: `safe/include/png.h`, `safe/include/pngconf.h`, `safe/include/pnglibconf.h`, `safe/abi/exports.txt`, `safe/abi/libpng.vers`, and `safe/abi/install-layout.txt` should only change for a proven ABI/API or install-layout issue and must be verified with the matching local checks.
- Every implementation phase must run a fresh full libpng validator suite into its own artifact root and write `validator.exit-code`; no phase may fall back to an earlier artifact root for its own verifier checks.
- Every package-building phase and verifier must run the Debian build-product cleanup and commit gate before accepting the commit.

# Verification Phases

## `check-validator-baseline-software-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-validator-baseline`
- Purpose: verify the validator checkout, local override package root, package build, initial validator artifact root, failure inventory, and report baseline.
- Commands:
  - `git status --short --branch`
  - `git -C validator rev-parse HEAD`
  - `cd validator && make unit && make check-testcases`
  - `find validator-overrides/libpng -maxdepth 1 -type f -name '*.deb' -printf '%f\n' | LC_ALL=C sort`
  - Run this baseline parser:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-initial")
    results_dir = root / "results/libpng"
    summary_path = results_dir / "summary.json"
    if not summary_path.is_file():
        raise SystemExit(f"missing summary: {summary_path}")
    summary = json.loads(summary_path.read_text())
    for key in ("cases", "source_cases", "usage_cases", "passed", "failed", "casts"):
        if not isinstance(summary.get(key), int):
            raise SystemExit(f"summary {key} must be an integer")
    if summary.get("library") != "libpng" or summary.get("mode") != "original":
        raise SystemExit(f"unexpected summary library/mode: {summary}")
    if summary["cases"] <= 0:
        raise SystemExit("baseline validator discovered no libpng cases")
    if summary["source_cases"] + summary["usage_cases"] != summary["cases"]:
        raise SystemExit(f"source+usage count mismatch: {summary}")
    result_paths = sorted(p for p in results_dir.glob("*.json") if p.name != "summary.json")
    if len(result_paths) != summary["cases"]:
        raise SystemExit(f"expected {summary['cases']} result files, found {len(result_paths)}")
    failures = []
    for path in result_paths:
        result = json.loads(path.read_text())
        if result.get("testcase_id") != path.stem:
            raise SystemExit(f"testcase_id/path mismatch in {path}")
        if result.get("status") != "passed":
            failures.append((path.stem, result.get("status"), result.get("exit_code"), result.get("log_path")))
    print(f"baseline summary: {summary}")
    if failures:
        print("baseline failures:")
        for testcase_id, status, exit_code, log_path in failures:
            print(f"  {testcase_id}: status={status} exit_code={exit_code} log={log_path}")
    PY
    ```

  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-initial`.
  - Inline and run the Debian build-product commit gate below.
  - `rg -n "Validator commit|Initial run|Failures|validator/artifacts/libpng-safe-initial" validator-report.md`

## `check-validator-baseline-senior-tester`

- Type: `check`
- Fixed `bounce_target`: `impl-validator-baseline`
- Purpose: review use of the validator README local override contract and confirm validator tests are unchanged.
- Commands:
  - `git -C validator status --short`
  - `test -z "$(git -C validator status --short -- tests repositories.yml tools test.sh)"`
  - `git -C validator diff --exit-code -- tests repositories.yml tools test.sh`
  - Inline and run the phase full-suite artifact gate below with `ROOT_NAME=libpng-safe-initial`.
  - Inline and run the Debian build-product commit gate below.
  - Run this artifact/cast review:

    ```bash
    python3 - <<'PY'
    import json
    from pathlib import Path

    root = Path("validator/artifacts/libpng-safe-initial")
    results_dir = root / "results/libpng"
    result_paths = sorted(p for p in results_dir.glob("*.json") if p.name != "summary.json")
    if not result_paths:
        raise SystemExit(f"no result JSON files under {results_dir}")
    missing = []
    bad_overrides = []
    bad_casts = []
    for path in result_paths:
        result = json.loads(path.read_text())
        testcase_id = result.get("testcase_id", path.stem)
        for rel_field in ("result_path", "log_path"):
            rel = result.get(rel_field)
            if not isinstance(rel, str) or not (root / rel).is_file():
                missing.append(f"{testcase_id} missing {rel_field}: {rel}")
        if result.get("override_debs_installed") is not True:
            bad_overrides.append(testcase_id)
        cast_path = result.get("cast_path")
        if not isinstance(cast_path, str) or not (root / cast_path).is_file():
            bad_casts.append(testcase_id)
    if missing:
        raise SystemExit("\n".join(missing))
    if bad_overrides:
        raise SystemExit("override packages were not installed for: " + ", ".join(bad_overrides))
    if bad_casts:
        raise SystemExit("missing casts for: " + ", ".join(bad_casts))
    print(f"verified {len(result_paths)} baseline results with installed overrides and casts")
    PY
    ```

  - `rg -n "mode original|override-deb-root|libpng-safe-initial|validator-overrides/libpng" validator-report.md`

## Shared verifier command blocks

The following command blocks must be inlined into each checker prompt that references them.

```bash
ROOT_NAME=libpng-safe-initial python3 - <<'PY'
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

- Validator checkout is cloned or fast-forward updated, with its post-update commit recorded.
- Validator unit and testcase metadata checks pass.
- Packages are rebuilt from local `safe/`, and override packages are refreshed.
- `validator-case-inventory.json` matches current libpng validator discovery.
- `validator/artifacts/libpng-safe-initial/` contains a complete full-suite run with exit code, per-case JSON, logs, and casts.
- `validator-report.md` records the initial summary and classified failures.
- Validator suite files remain locally unmodified.
- Debian build products are not staged or tracked.

# Git Commit Requirement

The implementer must commit all tracked work for this phase to git before yielding.
