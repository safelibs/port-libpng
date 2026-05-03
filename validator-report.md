# Validator Report: libpng-safe Initial

## Summary

- Phase: `impl-validator-baseline`.
- Repository root: `/home/yans/safelibs/pipeline/ports/port-libpng`.
- Validator checkout: `validator/` (already at upstream `main` head; `git pull --ff-only` reported "Already up to date.").
- Validator commit: `87b321fe728340d6fc6dd2f638583cca82c667c3`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/`.
- Initial artifact root: `validator/artifacts/libpng-safe-initial/`.
- Initial validator exit code: `0`.
- Initial result: 175/175 passed, 0 failed, 175 casts recorded.
- Inventory match: 175 total cases, 5 source cases, 170 usage cases, matching `validator-case-inventory.json`.
- Validator suite changes: none.

## Commands Executed

Validator update, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
git -C validator pull --ff-only
git -C validator rev-parse HEAD
```

Validator tooling checks, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

```bash
make unit
make check-testcases
```

Initial package build and package gate, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
```

Initial override refresh, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
rm -f validator-overrides/libpng/*.deb
cp libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
```

Initial validator run, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

```bash
rm -rf artifacts/libpng-safe-initial
mkdir -p artifacts/libpng-safe-initial
set +e
bash test.sh \
  --config repositories.yml \
  --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-initial" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
status=$?
printf '%s\n' "$status" > artifacts/libpng-safe-initial/validator.exit-code
exit "$status"
```

Initial cleanup command, run before commit from `/home/yans/safelibs/pipeline/ports/port-libpng`:

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

## Package Artifacts

Root package artifact SHA-256 values:

| SHA-256 | Artifact |
| --- | --- |
| `98a9add5589904c1182687a278a513d43156a87359201437b859cd5418278090` | `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `634fe83cc53e8eb8905cdddff28d3dab448d540d99b8bcf48929802fe932e350` | `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `a915d48037d9d14a858ea60566f71989788e1f3bb0a2e5754eac385cd9112d85` | `libpng16-16t64-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `1c567d67fbc99e6a32015d434895edb5bd2bbcdeb810a749d80e5f4745dcce4b` | `libpng-tools-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `e7d07f272fc8c02d98a39ed4ed1b3d1e69d71fc386d08acbe24eb58116a85cf4` | `libpng16-16-udeb_1.6.43-5ubuntu0.5+safelibs1_amd64.udeb` |
| `74c85e838b317fbb4584758606dc33b187c58e46a9a7d804c89206556b9c0ea3` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `400e318d8d71ba6c4a6768451a5647e546a3b65e05ebc1678db0c9aa66a6073e` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |
| `568780a251cb52f9bf127d3de3e47d50fd1006fd1ed57ceae3f04e687c9768b8` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.dsc` |
| `77d167c916c6c262a2a89b0652a0de5c707ba5dd56ae65865461ea640e7d86c7` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz` |
| `40eb4fc7fa7481bd253130ed52c4e1c6f48cdd22ec875d89b63810882f8bf89f` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.tar.xz` |
| `245573d767b5374b12e0d261b69d38c48236b15581c5cf3de8b46caa494e4ba5` | `libpng1.6_1.6.43.orig.tar.xz` |
| `cc9a5724c9332a309d8d66457afffba6678322f3ba3a610a16b98a0c93c6d756` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.buildinfo` |
| `be940246dcb971516a196b7eb2a6b92814a5feaac1a7800c7a350016936633ba` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.changes` |

Validator override SHA-256 values:

| SHA-256 | Override artifact |
| --- | --- |
| `98a9add5589904c1182687a278a513d43156a87359201437b859cd5418278090` | `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `634fe83cc53e8eb8905cdddff28d3dab448d540d99b8bcf48929802fe932e350` | `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |

## Initial Validator Results

- Summary JSON: `validator/artifacts/libpng-safe-initial/results/libpng/summary.json`.
- Validator exit code file: `validator/artifacts/libpng-safe-initial/validator.exit-code`.
- Result JSON files: 175 testcase files plus `summary.json`.
- Testcase logs: 175 testcase logs plus `docker-build.log`.
- Cast files: 175, one per testcase.
- Artifact consistency: every testcase ID from `validator-case-inventory.json` has a matching result JSON, testcase log, and cast.

Initial summary:

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 175,
  "source_cases": 5,
  "usage_cases": 170,
  "passed": 175,
  "failed": 0,
  "casts": 175,
  "duration_seconds": 0.0
}
```

## Failures And Classification

Per the phase failure mapping, failing testcase IDs are classified as
follows. Source/API covers `chunk-metadata-inspection` and
`read-write-c-api-smoke`; CLI/source fixtures cover `malformed-png-rejection`,
`palette-fixture-handling`, and `pngfix-fixture-handling`; Netpbm usage covers
testcase IDs starting with `usage-netpbm-`; pngquant usage covers testcase IDs
starting with `usage-pngquant-`; everything else falls into Other/catch-all.

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

No failing testcases were observed in this initial run; the initial validator
exit code is `0`. No validator suite files were modified.

## Source/API Phase

Phase: `impl-source-api-fixes`. Both source/API testcases
(`chunk-metadata-inspection` and `read-write-c-api-smoke`) passed in the
Phase 1 baseline at `validator/artifacts/libpng-safe-initial/`. No source/API
failures exist in the current run, so no `safe/src/` changes, no new local C
reproducers, and no validator suite edits were required for this phase. The
existing source/API regressions are already covered by the local check
batteries: `safe/tools/check-core-smoke.sh`,
`safe/tools/check-read-core.sh`, and `safe/tools/check-read-transforms.sh`.
Source files changed in this phase: none.

Validator scripts referenced (read-only inputs):

- `validator/tests/libpng/tests/cases/source/chunk-metadata-inspection.sh`
- `validator/tests/libpng/tests/cases/source/read-write-c-api-smoke.sh`

Source/API rerun commands, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
rm -f validator-overrides/libpng/*.deb
cp libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
( cd safe && cargo fmt --check && cargo test --quiet )
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
safe/tools/check-exports.sh
safe/tools/check-headers.sh
```

Source/API validator rerun, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

```bash
rm -rf artifacts/libpng-safe-source-api
mkdir -p artifacts/libpng-safe-source-api
set +e
bash test.sh \
  --config repositories.yml \
  --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-source-api" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
status=$?
printf '%s\n' "$status" > artifacts/libpng-safe-source-api/validator.exit-code
exit "$status"
```

Source/API validator results:

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 175,
  "source_cases": 5,
  "usage_cases": 170,
  "passed": 175,
  "failed": 0,
  "casts": 175,
  "duration_seconds": 0.0
}
```

- Source/API summary JSON: `validator/artifacts/libpng-safe-source-api/results/libpng/summary.json`.
- Source/API exit code file: `validator/artifacts/libpng-safe-source-api/validator.exit-code` (`0`).
- `chunk-metadata-inspection`: `passed`.
- `read-write-c-api-smoke`: `passed`.
- Package SHA-256s match the Phase 1 baseline; rebuild is reproducible.

Source/API rerun failures by class:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

## Artifact Gates

The full-suite artifact gate was satisfied for both phase roots:

| Artifact root | Cases | Results | Logs | Casts | Exit code |
| --- | ---: | ---: | ---: | ---: | ---: |
| `validator/artifacts/libpng-safe-initial/` | 175 | 175 | 175 | 175 | 0 |
| `validator/artifacts/libpng-safe-source-api/` | 175 | 175 | 175 | 175 | 0 |

## Proof And Exceptions

`validator-case-inventory.json` records `proof_rejects_original_override: true`.
At validator commit `87b321fe728340d6fc6dd2f638583cca82c667c3`, proof
generation rejects original-mode result JSON when `override_debs_installed` is
`true`. Acceptance evidence for this initial phase is the per-case result JSON,
testcase logs, casts, package hashes, the artifact gate above, and the recorded
validator exit code.

Validator Bug Exceptions: none
