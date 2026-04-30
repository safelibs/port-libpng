# Validator Report: libpng-safe Final

## Summary

- Phase: `impl-final-clean-validation-and-report`.
- Repository root: `/home/yans/safelibs/pipeline/ports/port-libpng`.
- Safe port baseline before this phase: `cce03607a6dc1269bbaedd2cb25ee17ac85a9616`.
- Validator checkout: `validator/`.
- Validator commit: `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/`.
- Final artifact root: `validator/artifacts/libpng-safe-final/`.
- Final validator exit code: `0`.
- Final result: 135/135 passed, 0 failed, 135 casts recorded.
- Inventory match: 135 total cases, 5 source cases, 130 usage cases, matching `validator-case-inventory.json`.
- Validator suite changes: none.

## Commands Executed

Local verification battery, run from `/home/yans/safelibs/pipeline/ports/port-libpng`:

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

Final package build and package gate:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
```

Final override refresh:

```bash
cp libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
cp libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb validator-overrides/libpng/
```

Final validator run, run from `/home/yans/safelibs/pipeline/ports/port-libpng/validator`:

```bash
rm -rf artifacts/libpng-safe-final
mkdir -p artifacts/libpng-safe-final
set +e
bash test.sh \
  --config repositories.yml \
  --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-final" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
status=$?
printf '%s\n' "$status" > artifacts/libpng-safe-final/validator.exit-code
exit "$status"
```

Final cleanup command:

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
| `392eb0ea6445677ec3954013362ba1bb594f09ed40b14190098b318c487cc1a9` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `9006ea70acb6ac634dbe640739600f5faea2fdad6262a5666afb46d60561b6cd` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |
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

## Final Validator Results

- Summary JSON: `validator/artifacts/libpng-safe-final/results/libpng/summary.json`.
- Validator exit code file: `validator/artifacts/libpng-safe-final/validator.exit-code`.
- Result JSON files: 135 testcase files plus `summary.json`.
- Testcase logs: 135 testcase logs plus `docker-build.log`.
- Cast files: 135, one per testcase.
- Artifact consistency: every testcase ID from `validator-case-inventory.json` has a matching result JSON, testcase log, and cast.
- Result consistency: every per-case result has `status: passed`, `exit_code: 0`, `mode: original`, and `override_debs_installed: true`.

Final summary:

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 135,
  "source_cases": 5,
  "usage_cases": 130,
  "passed": 135,
  "failed": 0,
  "casts": 135,
  "duration_seconds": 0.0
}
```

## Artifact Gates

The full-suite artifact gate passed for every required phase root:

| Artifact root | Cases | Results | Logs | Casts | Exit code |
| --- | ---: | ---: | ---: | ---: | ---: |
| `validator/artifacts/libpng-safe-initial/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-source-api/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-cli-source/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-usage-netpbm/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-usage-pngquant/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-catch-all/` | 135 | 135 | 135 | 135 | 0 |
| `validator/artifacts/libpng-safe-final/` | 135 | 135 | 135 | 135 | 0 |

## Failures And Fixes

Failure classification after the final validator run:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

Implementation phase commits consumed by this final pass:

| Commit | Phase |
| --- | --- |
| `a45ebe1` | Initial libpng-safe validator run |
| `3f5ece7` | Source/API validator phase |
| `18842f6` | CLI/source validator phase |
| `9ff9d56` | Netpbm usage validator phase |
| `2cc858a` | pngquant usage validator phase |
| `cce0360` | Catch-all validator phase |

No safe source, validator suite, or regression-test changes were needed in the
final clean-validation phase. The existing CVE, dependent, upstream, read, write,
header, export, link, install-surface, build-layout, and package gates all
passed before the final package rebuild and validator run.

## Proof And Exceptions

`validator-case-inventory.json` records `proof_rejects_original_override: true`.
At validator commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`, proof
generation rejects original-mode result JSON when `override_debs_installed` is
`true`. Per the phase instructions, the proof verifier was not rerun or modified;
acceptance evidence is the final result JSON, logs, casts, package hashes,
artifact gate, and validator exit code. This proof-tooling limitation is not a
validator testcase bug exception because the final validator summary has zero
failing testcases.

Validator Bug Exceptions: none
