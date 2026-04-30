# Validator Report: libpng-safe Final Clean Validation

## Summary

- Phase: `impl-final-clean-validation-and-report`.
- Validator checkout: `validator/`.
- Validator commit: `cc99047419226144eec3c1ab87873052bd9abedc`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/`.
- Final unfiltered artifact root: `validator/artifacts/libpng-safe-final/`.
- Final validator exit code: `0`.
- Final result: 105/105 passed, 0 failed, 105 casts recorded.
- Inventory match: 105 total cases, 5 source cases, 100 usage cases, matching `validator-case-inventory.json`.
- Validator bug exceptions: none. No filtered exception run was needed.
- Validator source changes: none; the validator checkout is clean apart from ignored generated artifacts.

## Checks Executed

All commands below completed with exit code `0` in this final phase.

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
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
cd validator && bash test.sh --config repositories.yml --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-final" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
```

The package rebuild refreshed the binary build metadata. `safe/tools/check-package-artifacts.sh` confirmed that the package artifacts match the current safe packaging tree, the safe source snapshot tar matches the tracked safe tree, and the `libpng-dev` examples are preserved.

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
| `7818e89e54a7f1697ade4dd9db6ab64af62ad4f7f819bbd4ce7a32f4cf37ab21` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.dsc` |
| `a4769982b43c3b071cce54a565b27f6e52a26ceedaca252585a61f0d0ef647f2` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz` |
| `110429cefa2093b1080296a9634296059818553aaa3d00229b0923edfc9bedd7` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.tar.xz` |
| `245573d767b5374b12e0d261b69d38c48236b15581c5cf3de8b46caa494e4ba5` | `libpng1.6_1.6.43.orig.tar.xz` |
| `392eb0ea6445677ec3954013362ba1bb594f09ed40b14190098b318c487cc1a9` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `9006ea70acb6ac634dbe640739600f5faea2fdad6262a5666afb46d60561b6cd` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |
| `daa4459755f48ed01f8a72149dd534bbc7ac3f0cba5d74ca502c399e0677da8b` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.buildinfo` |
| `a84204147c1c70398206729012941d925c2cda3b0dcf4f1154df9ec396e53ce0` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.changes` |

Local validator override SHA-256 values:

| SHA-256 | Override artifact |
| --- | --- |
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |

## Final Validator Results

- Summary JSON: `validator/artifacts/libpng-safe-final/results/libpng/summary.json`.
- Validator exit code file: `validator/artifacts/libpng-safe-final/validator.exit-code`.
- Result JSON files: 105 testcase files plus `summary.json`.
- Cast files: 105, one per testcase.
- Testcase logs: 105, one per testcase, plus `docker-build.log`.
- Source cases: 5/5 passed.
- Usage cases: 100/100 passed.
- Artifact consistency check: every testcase ID from `validator-case-inventory.json` has a matching result JSON, cast, and testcase log; every per-case result has `status: passed`, `exit_code: 0`, `mode: original`, and `override_debs_installed: true`.

Final summary:

```json
{
  "schema_version": 2,
  "library": "libpng",
  "mode": "original",
  "cases": 105,
  "source_cases": 5,
  "usage_cases": 100,
  "passed": 105,
  "failed": 0,
  "casts": 105,
  "duration_seconds": 0.0
}
```

## Failures Found

- Initial validator setup run (`b48cd1a`) found 28 usage-case failures: 26 Netpbm client failures and 2 pngquant client failures.
- Source/API assignment reruns found no active validator source-case failures.
- CLI/source assignment reruns found no active source fixture, `pngfix`, malformed input, palette, package-installation, or missing-binary failures.
- Final unfiltered run found no failures.

## Fixes Applied And Regression Tests

| Commit | Scope | Regression test | Verification command |
| --- | --- | --- | --- |
| `419ffdd` | Fixed simplified 8-bit non-linear read/write gamma handling so adding opaque alpha preserves encoded grayscale samples. | `safe/tests/read-transforms/simplified_read_driver.c` compares `PNG_FORMAT_GRAY` and `PNG_FORMAT_GA` reads of `original/contrib/testpngs/gray-2-1.8.png`; `safe/tools/check-read-transforms.sh` runs the fixture. | `safe/tools/check-read-transforms.sh`; `safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha pngstest-large-stride`; `safe/tools/run-upstream-tests.sh`. |
| `ca94b46` | Fixed write runtime row handling so clients using `png_set_packing` pass unpacked row input and receive correctly packed PNG output. | `safe/tests/dependents/write_packing_indices.c` writes a 3x3 4-bit palette PNG with unpacked indices plus `png_set_packing`, then reads back the exact index grid. | `safe/tools/run-dependent-regressions.sh`; full validator reruns under `validator/artifacts/libpng-safe-usage-client/`, `validator/artifacts/libpng-safe-catch-all/`, and `validator/artifacts/libpng-safe-final/`. |
| `52c42ec` | Confirmed assigned CLI/source validator cases already passed; no `safe/` compatibility fix was needed. | None added because no failing behavior was present to reproduce. | `safe/tools/check-examples-and-tools.sh`; `safe/tools/run-upstream-tests.sh`; `safe/tests/upstream/pngfix.sh`; full validator rerun under `validator/artifacts/libpng-safe-cli-source/`. |
| `dd6c8e3` | Confirmed no residual validator failures remained after the usage-client fix. | None added because the catch-all rerun was already clean. | Full local battery and full validator rerun under `validator/artifacts/libpng-safe-catch-all/`. |

Packaging metadata-only refreshes were committed separately where needed, including `a7830d2` and this final phase's binary build metadata refresh.

## Validator Bug Exceptions

No libpng validator testcase was classified as a validator bug, no testcase was skipped, and no `validator-exception-tests/` or `validator/artifacts/libpng-safe-final-filtered/` output was needed.

The only validator tooling limitation recorded for this workflow is proof/site generation for original-mode local override results, described below. It is not a testcase exception and was not used to skip any libpng matrix check.

## Local Override Proof Tooling

`validator-case-inventory.json` records `proof_rejects_original_override: true` for validator commit `cc99047419226144eec3c1ab87873052bd9abedc`. The checked-out proof tooling rejects original-mode result JSON when `override_debs_installed` is `true`; this validation intentionally installs local override `.deb` packages, and the final per-case result JSON records `override_debs_installed: true`.

Therefore proof and site targets were intentionally not run for the final local-override validation. Acceptance for this phase is based on the final result JSON, summary JSON, casts, logs, validator exit code, package hashes, and this report.
