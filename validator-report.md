# Validator Report: libpng-safe Initial

## Summary

- Phase: `impl-validator-baseline`.
- Repository root: `/home/yans/safelibs/pipeline/ports/port-libpng`.
- Safe port baseline: `8f31c801016bf7822cfd8172e1bf0f8dab1ca1e4`.
- Validator checkout: `validator/` (fast-forward updated from `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`).
- Validator commit: `87b321fe728340d6fc6dd2f638583cca82c667c3`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/`.
- Initial artifact root: `validator/artifacts/libpng-safe-initial/`.
- Initial validator exit code: `1`.
- Initial result: 173/175 passed, 2 failed, 175 casts recorded.
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
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `0b0697d920eba71496e56b3be1c175be60b7df2835ea7f5f3de7ef933db82b6e` | `libpng16-16t64-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `1c567d67fbc99e6a32015d434895edb5bd2bbcdeb810a749d80e5f4745dcce4b` | `libpng-tools-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `f07558cabbc0cf6d369cb695d040dfdb207326d0ac5b0be1eabb7575e34fdc97` | `libpng16-16-udeb_1.6.43-5ubuntu0.5+safelibs1_amd64.udeb` |
| `b03d894280a765e72b5e1720e9424e8ca7f7a0d6927a9376ad45de1c44023598` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `4f0b1a85aa7c38f4a4cb2fd34349951cd07d70276a8740777710659043daf503` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |
| `e4cbab99737e5e2bc4b7a402e5f292db0ad3342b40029310be5745cc0ac8cb74` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.dsc` |
| `e1771f5eb16560c498ea7327663ff07cb1768a1d84fc461048bf9ad04d1545c8` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz` |
| `ec6a4c651d068f83c4c41527f737ab5bab70a78ed01ddf3c3632ca720510a9bc` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.tar.xz` |
| `245573d767b5374b12e0d261b69d38c48236b15581c5cf3de8b46caa494e4ba5` | `libpng1.6_1.6.43.orig.tar.xz` |
| `c836307b908defdd17c1e944ec024a72806682ff75e5537559ba91668f52366f` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.buildinfo` |
| `dfd534231d2cf5f42a449a9501fbc2018ea0d55b335ec667238bf33dd2da7589` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.changes` |

Validator override SHA-256 values:

| SHA-256 | Override artifact |
| --- | --- |
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
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
  "passed": 173,
  "failed": 2,
  "casts": 175,
  "duration_seconds": 0.0
}
```

## Failures And Classification

Per the phase failure mapping, every failing testcase ID is classified as
follows. Source/API covers `chunk-metadata-inspection` and
`read-write-c-api-smoke`; CLI/source fixtures cover `malformed-png-rejection`,
`palette-fixture-handling`, and `pngfix-fixture-handling`; Netpbm usage covers
testcase IDs starting with `usage-netpbm-`; pngquant usage covers testcase IDs
starting with `usage-pngquant-`; everything else falls into Other/catch-all.

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | `usage-netpbm-pamtopng-text-chunk-png`, `usage-netpbm-pnmtopng-transparent-color-png` |
| pngquant usage | none |
| Other/catch-all | none |

Failure observations from per-case logs under
`validator/artifacts/libpng-safe-initial/logs/libpng/`:

- `usage-netpbm-pamtopng-text-chunk-png`: pamtopng emitted only `zTXt` chunks
  (`['IHDR', 'zTXt', 'zTXt', 'zTXt', 'zTXt', 'IDAT', 'IEND']`); the testcase
  expected at least two `tEXt` chunks. Exit code 1.
- `usage-netpbm-pnmtopng-transparent-color-png`: `pngtopam -alphapam` produced
  a segfault (exit code 139) when reading the safe-encoded PNG.

These failures will be addressed in subsequent implementation phases. No
validator suite files were modified.

## Artifact Gates

The full-suite artifact gate was satisfied for the initial phase root:

| Artifact root | Cases | Results | Logs | Casts | Exit code |
| --- | ---: | ---: | ---: | ---: | ---: |
| `validator/artifacts/libpng-safe-initial/` | 175 | 175 | 175 | 175 | 1 |
| `validator/artifacts/libpng-safe-source-api/` | 175 | 175 | 175 | 175 | 1 |
| `validator/artifacts/libpng-safe-cli-source/` | 175 | 175 | 175 | 175 | 1 |

## Source/API Phase: `impl-source-api-validator-failures`

- Phase: `impl-source-api-validator-failures`.
- Source/API testcases in scope: `chunk-metadata-inspection`, `read-write-c-api-smoke`.
- Source/API result in baseline `validator/artifacts/libpng-safe-initial/`: both passed.
- Source/API result in this phase root `validator/artifacts/libpng-safe-source-api/`: both passed.
- No source/API failures in current run; this phase is documentation-only with a fresh full-suite rerun.
- Reproducer paths: none added (no source/API failures to reproduce).
- Safe source files changed: none.
- Validator suite changes: none.

Phase commands, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
cd /home/yans/safelibs/pipeline/ports/port-libpng
safe/tools/check-package-artifacts.sh
rm -f validator-overrides/libpng/*.deb
cp libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
safe/tools/check-headers.sh
safe/tools/check-exports.sh
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
```

Source/API phase validator run, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

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

Source/API phase validator summary (`validator/artifacts/libpng-safe-source-api/results/libpng/summary.json`):

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 175,
  "source_cases": 5,
  "usage_cases": 170,
  "passed": 173,
  "failed": 2,
  "casts": 175,
  "duration_seconds": 0.0
}
```

Source/API phase validator exit code (`validator/artifacts/libpng-safe-source-api/validator.exit-code`): `1`.

Remaining failures by class for this phase root:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | `usage-netpbm-pamtopng-text-chunk-png`, `usage-netpbm-pnmtopng-transparent-color-png` |
| pngquant usage | none |
| Other/catch-all | none |

Package and override SHA-256 values are unchanged from the baseline rebuild
recorded above; the source/API phase rebuild is byte-identical to the
`impl-validator-baseline` rebuild.

## CLI/Source Phase: `impl-cli-source-validator-failures`

- Phase: `impl-cli-source-validator-failures`.
- CLI/source testcases in scope: `malformed-png-rejection`, `palette-fixture-handling`, `pngfix-fixture-handling`.
- CLI/source results in prior phase root `validator/artifacts/libpng-safe-source-api/`: all three passed.
- CLI/source results in this phase root `validator/artifacts/libpng-safe-cli-source/`: all three passed.
- No CLI/source failures in current run; this phase is documentation-only with a fresh full-suite rerun.
- Reproducer paths: none added (no CLI/source failures to reproduce).
- Safe source files changed: none.
- Packaged tool files changed: none.
- Validator suite changes: none.

Phase commands, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
cd /home/yans/safelibs/pipeline/ports/port-libpng
safe/tools/check-package-artifacts.sh
rm -f validator-overrides/libpng/*.deb
cp libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
```

CLI/source phase validator run, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

```bash
rm -rf artifacts/libpng-safe-cli-source
mkdir -p artifacts/libpng-safe-cli-source
set +e
bash test.sh \
  --config repositories.yml \
  --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-cli-source" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
status=$?
printf '%s\n' "$status" > artifacts/libpng-safe-cli-source/validator.exit-code
exit "$status"
```

CLI/source phase validator summary (`validator/artifacts/libpng-safe-cli-source/results/libpng/summary.json`):

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 175,
  "source_cases": 5,
  "usage_cases": 170,
  "passed": 173,
  "failed": 2,
  "casts": 175,
  "duration_seconds": 0.0
}
```

CLI/source phase validator exit code (`validator/artifacts/libpng-safe-cli-source/validator.exit-code`): `1`.

Remaining failures by class for this phase root:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | `usage-netpbm-pamtopng-text-chunk-png`, `usage-netpbm-pnmtopng-transparent-color-png` |
| pngquant usage | none |
| Other/catch-all | none |

Package and override SHA-256 values are unchanged from the baseline rebuild
recorded above; the CLI/source phase rebuild is byte-identical to the
`impl-validator-baseline` rebuild.

## Proof And Exceptions

`validator-case-inventory.json` records `proof_rejects_original_override: true`.
At validator commit `87b321fe728340d6fc6dd2f638583cca82c667c3`, proof
generation rejects original-mode result JSON when `override_debs_installed` is
`true`. Acceptance evidence for this initial phase is the per-case result JSON,
testcase logs, casts, package hashes, the artifact gate above, and the recorded
validator exit code. The remaining failing testcases are tracked for follow-up
implementation phases.

Validator Bug Exceptions: none
