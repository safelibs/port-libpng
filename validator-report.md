# Validator Report: libpng-safe Initial Run

## Summary

- Phase: `impl-validator-setup-and-initial-run`.
- Validator commit: `cc99047419226144eec3c1ab87873052bd9abedc`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/*.deb`.
- Initial matrix result: exit code `1`, 77/105 passed, 28 failed, 105 casts recorded.
- Package artifact validation passed; no package rebuild and no `safe/` code changes were made in this phase.
- All source-facing libpng cases passed. The 28 failures are usage cases and are treated as libpng-safe compatibility regressions for later implementation phases.

## Validator Checkout

- Checkout path: `validator/`.
- Remote: `https://github.com/safelibs/validator`.
- Commit: `cc99047419226144eec3c1ab87873052bd9abedc`.
- README confirms package override flow, not direct source-path validation: local override roots are laid out as `<override-deb-root>/<library>/*.deb`.
- `repositories.yml` and `tests/libpng/testcases.yml` map `libpng` to `libpng16-16t64`, `libpng-dev`, and `libpng-tools`.

## Case Inventory

- Library: `libpng`.
- Apt packages: `libpng16-16t64`, `libpng-dev`, `libpng-tools`.
- Total cases: 105.
- Source cases: 5.
- Usage cases: 100.
- Source case IDs: `chunk-metadata-inspection`, `malformed-png-rejection`, `palette-fixture-handling`, `pngfix-fixture-handling`, `read-write-c-api-smoke`.
- Usage case IDs:
  - `usage-netpbm-batch11-pamdeinterlace-height`, `usage-netpbm-batch11-pamdice-tiles`, `usage-netpbm-batch11-pamstretch-double`, `usage-netpbm-batch11-pamthreshold-bw`, `usage-netpbm-batch11-pngtopam-pamfile`, `usage-netpbm-batch11-pnmarith-add`, `usage-netpbm-batch11-pnmgamma-shape`, `usage-netpbm-batch11-pnmnorm-shape`
  - `usage-netpbm-batch11-pnmrotate-right-angle`, `usage-netpbm-batch11-pnmshear-width`, `usage-netpbm-pamarith-add-png`, `usage-netpbm-pamchannel-blue-generated-png`, `usage-netpbm-pamchannel-blue-png`, `usage-netpbm-pamchannel-green-png`, `usage-netpbm-pamchannel-green-png-generated`, `usage-netpbm-pamchannel-red-generated-png`
  - `usage-netpbm-pamchannel-red-png`, `usage-netpbm-pamchannel-red-png-generated`, `usage-netpbm-pamfile-png`, `usage-netpbm-pamflip-ccw-grayscale-png`, `usage-netpbm-pamflip-cw-png-generated`, `usage-netpbm-pamflip-png`, `usage-netpbm-pamtopnm-roundtrip-png`, `usage-netpbm-pngtopam`
  - `usage-netpbm-pngtopnm`, `usage-netpbm-pnmcat-leftright-png`, `usage-netpbm-pnmcat-leftright-png-generated`, `usage-netpbm-pnmcat-topbottom-png`, `usage-netpbm-pnmcat-topbottom-png-generated`, `usage-netpbm-pnmcrop-border-png`, `usage-netpbm-pnmcut-bottom-row-png`, `usage-netpbm-pnmcut-corner-png`
  - `usage-netpbm-pnmcut-middle-column-png`, `usage-netpbm-pnmcut-middle-row-png`, `usage-netpbm-pnmcut-png`, `usage-netpbm-pnmdepth-15-png-generated`, `usage-netpbm-pnmdepth-63-png-generated`, `usage-netpbm-pnmdepth-png`, `usage-netpbm-pnmfile-png`, `usage-netpbm-pnmfile-roundtrip-png`
  - `usage-netpbm-pnmflip-leftright-generated-png`, `usage-netpbm-pnmflip-leftright-png-generated`, `usage-netpbm-pnmflip-png`, `usage-netpbm-pnmflip-r180-png`, `usage-netpbm-pnmflip-rotate180-rgb-png`, `usage-netpbm-pnmflip-topbottom-png`, `usage-netpbm-pnmflip-topbottom-png-generated`, `usage-netpbm-pnmflip-transpose-png`
  - `usage-netpbm-pnmgamma-png`, `usage-netpbm-pnminvert-png`, `usage-netpbm-pnminvert-png-generated`, `usage-netpbm-pnminvert-rgb-png-roundtrip`, `usage-netpbm-pnmpaste-png`, `usage-netpbm-pnmpsnr-png`, `usage-netpbm-pnmscale-double-png`, `usage-netpbm-pnmscale-double-png-generated`
  - `usage-netpbm-pnmscale-half-png`, `usage-netpbm-pnmscale-half-png-generated`, `usage-netpbm-pnmscale-png`, `usage-netpbm-pnmscale-triple-png`, `usage-netpbm-pnmsmooth-png`, `usage-netpbm-pnmtile-png`, `usage-netpbm-pnmtopng`, `usage-netpbm-roundtrip-png`
  - `usage-pngquant-cant-open-input-png`, `usage-pngquant-colors-eight-png`, `usage-pngquant-colors-eight-png-generated`, `usage-pngquant-colors-four-png`, `usage-pngquant-colors-four-png-generated`, `usage-pngquant-colors-sixteen-png`, `usage-pngquant-colors-three-png`, `usage-pngquant-colors-two-png`
  - `usage-pngquant-compress-png`, `usage-pngquant-ext-png`, `usage-pngquant-floyd-png`, `usage-pngquant-floyd-zero-png`, `usage-pngquant-iebug-png`, `usage-pngquant-map-palette-png`, `usage-pngquant-nofs-png`, `usage-pngquant-nofs-png-generated`
  - `usage-pngquant-posterize-one-png`, `usage-pngquant-posterize-png`, `usage-pngquant-posterize-two-png`, `usage-pngquant-quality-high-png`, `usage-pngquant-quality-low-png`, `usage-pngquant-quality-low-png-generated`, `usage-pngquant-quality-mid-png`, `usage-pngquant-quality-min-only-png`
  - `usage-pngquant-quality-png`, `usage-pngquant-quality-range-png`, `usage-pngquant-skip-if-larger-png`, `usage-pngquant-speed-eleven-png`, `usage-pngquant-speed-five-png`, `usage-pngquant-speed-one-png`, `usage-pngquant-speed-one-png-generated`, `usage-pngquant-speed-png`
  - `usage-pngquant-speed-three-png`, `usage-pngquant-strip-png`, `usage-pngquant-transbug-png`, `usage-pngquant-verbose-png`
- Phase 2 source API case IDs: `chunk-metadata-inspection`, `read-write-c-api-smoke`.
- Phase 3 source case IDs: `malformed-png-rejection`, `palette-fixture-handling`, `pngfix-fixture-handling`.
- Original-mode override supported: `true`.
- Proof rejects original override results: `true`.
- Testcase selector args: `[]`.
- Planned validator-bug exception skip method: `generated-dual-layout-tests-root`.

## Package Artifacts

- `safe/tools/check-package-artifacts.sh` exit code: `0`.
- Root package SHA-256:
  - `082c66f62ca76e9dbca80a3455940b5a55ed1189d1ea8dbcf13df50646b29a53`  `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
  - `c8e64907925b5ac67855774195bbe74c86dd67724887ad4646958e4aea449d1e`  `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
  - `69b42ea87177886a0e6ba458ea4768ab3fc165d60ee6e21a15f1d74e38ac6944`  `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- Override package SHA-256:
  - `082c66f62ca76e9dbca80a3455940b5a55ed1189d1ea8dbcf13df50646b29a53`  `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
  - `c8e64907925b5ac67855774195bbe74c86dd67724887ad4646958e4aea449d1e`  `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
  - `69b42ea87177886a0e6ba458ea4768ab3fc165d60ee6e21a15f1d74e38ac6944`  `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`

## Commands

```bash
if [ -d validator/.git ]; then git -C validator pull --ff-only; else git clone https://github.com/safelibs/validator validator; fi
git -C validator rev-parse HEAD
python3 <inventory script from phase source>
safe/tools/check-package-artifacts.sh
mkdir -p validator-overrides/libpng && install -m0644 libpng*.deb validator-overrides/libpng/
cd validator && make unit
cd validator && make check-testcases
cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root "$PWD/artifacts/libpng-safe-initial" --mode original --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides --library libpng --record-casts
cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root "$PWD/artifacts/libpng-safe-initial" --proof-output "$PWD/artifacts/libpng-safe-initial/proof/original-validation-proof.json" --library libpng --require-casts --min-source-cases 5 --min-usage-cases 100 --min-cases 105
```

- Command log root: `validator/artifacts/libpng-safe-initial/command-logs/`.
- `make unit` exit code: `2`. One validator self-test fails in `unit/test_render_site.py` because it expects `<strong>95 / 95</strong>` while the rendered fixture now reports `<strong>105 / 105</strong>`. The checkout was not modified.
- `make check-testcases` exit code: `0`.
- Validator matrix exit code: `1`; recorded in `validator/artifacts/libpng-safe-initial/validator.exit-code`.
- Original override proof verification exit code: `1`.

## Initial Results

- Summary JSON: `validator/artifacts/libpng-safe-initial/results/libpng/summary.json`.
- Result JSON count: 105 cases plus summary.
- Logs: `validator/artifacts/libpng-safe-initial/logs/libpng/`.
- Casts: `validator/artifacts/libpng-safe-initial/casts/libpng/` (105 casts).
- Source cases: 5/5 passed.
- Usage cases: 72/100 passed; 28 failed.

## Failure Classification

- Netpbm usage failures: 26. These logs show decoded or transformed pixel payload mismatches, wrong shapes, or unsupported generated Netpbm magic after reading/writing PNG through libpng-safe.
- Pngquant usage failures: 2. Both logs report `too few colors: 3`, consistent with color diversity loss in the image data presented through libpng-safe.
- No libpng source API validator testcase failed in this initial run.
- No per-testcase validator bug exception is claimed for the 28 libpng failures.

| Testcase | Client | Error | Log detail |
| --- | --- | --- | --- |
| `usage-netpbm-pamarith-add-png` | `netpbm` | `testcase command exited with status 1` | `File "/tmp/validator-tmp/pnm_assert.py", line 38, in <module> assert payload == expected, payload ^^^^^^^^^^^^^^^^^^^ AssertionError: [60, 60]` |
| `usage-netpbm-pamchannel-blue-generated-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [60, 60] != [30, 60]` |
| `usage-netpbm-pamchannel-green-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [50, 50] != [40, 50]` |
| `usage-netpbm-pamchannel-green-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload: [50, 50] != [20, 50]` |
| `usage-netpbm-pamchannel-red-generated-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [40, 40] != [10, 40]` |
| `usage-netpbm-pamchannel-red-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [10, 20]` |
| `usage-netpbm-pamflip-ccw-grayscale-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20, 20, 20] != [20, 40, 10, 30]` |
| `usage-netpbm-pamflip-cw-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20, 20, 20] != [30, 10, 40, 20]` |
| `usage-netpbm-pnmcat-leftright-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [10, 20]` |
| `usage-netpbm-pnmcat-topbottom-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [10, 20]` |
| `usage-netpbm-pnmcrop-border-png` | `netpbm` | `testcase command exited with status 1` | `unexpected shape` |
| `usage-netpbm-pnmcut-bottom-row-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [1, 7, 1] != [7, 8, 9]` |
| `usage-netpbm-pnmcut-corner-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload: [1, 1, 1, 1, 1, 1] != [6, 7, 10, 11, 14, 15]` |
| `usage-netpbm-pnmcut-middle-column-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [1, 4] != [2, 5]` |
| `usage-netpbm-pnmcut-middle-row-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [1, 3] != [3, 4]` |
| `usage-netpbm-pnmfile-roundtrip-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload: [0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0, 255] != [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]` |
| `usage-netpbm-pnmflip-leftright-generated-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [20, 10]` |
| `usage-netpbm-pnmflip-leftright-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [20, 10]` |
| `usage-netpbm-pnmflip-r180-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [1, 1, 1, 1] != [4, 3, 2, 1]` |
| `usage-netpbm-pnmflip-rotate180-rgb-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [40, 50, 60, 40, 50, 60, 40, 50, 60, 40, 50, 60] != [100, 110, 120, 70, 80, 90, 40, 50, 60, 10, 20, 30]` |
| `usage-netpbm-pnmflip-topbottom-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [20, 20] != [20, 10]` |
| `usage-netpbm-pnmflip-topbottom-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload: [20, 20, 20, 20] != [30, 40, 10, 20]` |
| `usage-netpbm-pnmflip-transpose-png` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [1, 1, 1, 1, 3, 5] != [1, 3, 5, 2, 4, 6]` |
| `usage-netpbm-pnminvert-png-generated` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [55, 55] != [245, 55]` |
| `usage-netpbm-pnminvert-png` | `netpbm` | `testcase command exited with status 1` | `unsupported netpbm magic: b'P4'` |
| `usage-netpbm-pnminvert-rgb-png-roundtrip` | `netpbm` | `testcase command exited with status 1` | `unexpected payload [245, 155, 55, 245, 155, 55] != [245, 155, 55, 225, 195, 165]` |
| `usage-pngquant-quality-high-png` | `pngquant` | `testcase command exited with status 1` | `too few colors: 3` |
| `usage-pngquant-speed-five-png` | `pngquant` | `testcase command exited with status 1` | `too few colors: 3` |

Deferred failure markers for later implementation phases:
- Deferred to catch-all: usage-netpbm-pamarith-add-png
- Deferred to catch-all: usage-netpbm-pamchannel-blue-generated-png
- Deferred to catch-all: usage-netpbm-pamchannel-green-png-generated
- Deferred to catch-all: usage-netpbm-pamchannel-green-png
- Deferred to catch-all: usage-netpbm-pamchannel-red-generated-png
- Deferred to catch-all: usage-netpbm-pamchannel-red-png-generated
- Deferred to catch-all: usage-netpbm-pamflip-ccw-grayscale-png
- Deferred to catch-all: usage-netpbm-pamflip-cw-png-generated
- Deferred to catch-all: usage-netpbm-pnmcat-leftright-png
- Deferred to catch-all: usage-netpbm-pnmcat-topbottom-png
- Deferred to catch-all: usage-netpbm-pnmcrop-border-png
- Deferred to catch-all: usage-netpbm-pnmcut-bottom-row-png
- Deferred to catch-all: usage-netpbm-pnmcut-corner-png
- Deferred to catch-all: usage-netpbm-pnmcut-middle-column-png
- Deferred to catch-all: usage-netpbm-pnmcut-middle-row-png
- Deferred to catch-all: usage-netpbm-pnmfile-roundtrip-png
- Deferred to catch-all: usage-netpbm-pnmflip-leftright-generated-png
- Deferred to catch-all: usage-netpbm-pnmflip-leftright-png-generated
- Deferred to catch-all: usage-netpbm-pnmflip-r180-png
- Deferred to catch-all: usage-netpbm-pnmflip-rotate180-rgb-png
- Deferred to catch-all: usage-netpbm-pnmflip-topbottom-png-generated
- Deferred to catch-all: usage-netpbm-pnmflip-topbottom-png
- Deferred to catch-all: usage-netpbm-pnmflip-transpose-png
- Deferred to catch-all: usage-netpbm-pnminvert-png-generated
- Deferred to catch-all: usage-netpbm-pnminvert-png
- Deferred to catch-all: usage-netpbm-pnminvert-rgb-png-roundtrip
- Deferred to catch-all: usage-pngquant-quality-high-png
- Deferred to catch-all: usage-pngquant-speed-five-png

## Fix Log

- No libpng-safe fixes were applied in this setup and initial-run phase.
- The next implementation phase should add minimal regression tests for the failing usage behavior before changing `safe/`.

## Validator Bug Exceptions

- Validator Bug Exception: <testcase-id>
- None applied to libpng validator testcases in this phase.
- Validator tooling issue documented: `make unit` has a stale rendered-count expectation in `unit/test_render_site.py` (`95 / 95` expected, `105 / 105` rendered). This was not used to skip any libpng matrix testcase.
- If a later phase proves a testcase-specific validator issue, use the planned skip method `generated-dual-layout-tests-root` and replace the placeholder marker above with `Validator Bug Exception: <testcase-id>` plus justification.

## Final Results

- Final result for this phase is the initial matrix result: no safe fixes were permitted or applied here.
- Validator matrix remains exit code `1` with 28 failing cases.
- Artifacts are preserved under `validator/artifacts/libpng-safe-initial/`.

## Local Override Proof Tooling

- `tools/proof.py` rejects original-mode results when `override_debs_installed` is true.
- Observed proof verification message: `override_debs_installed must be false for proof generation in /home/yans/safelibs/pipeline/ports/port-libpng/validator/artifacts/libpng-safe-initial/results/libpng/chunk-metadata-inspection.json`
- Because this run intentionally uses original mode with local package overrides, acceptance for this phase is based on result JSON, summary JSON, casts, logs, and this report rather than proof or site targets.

## Phase `impl-source-api-failures`

- Validator commit: `cc99047419226144eec3c1ab87873052bd9abedc`.
- Assignment source: `validator-case-inventory.json` Phase 2 source/API IDs `chunk-metadata-inspection` and `read-write-c-api-smoke`.
- Root cause: the assigned validator source/API cases still pass, but the verifier exposed a source-facing simplified C API regression through upstream `pngstest` wrappers. The simplified read runtime applied file `gAMA` transfer to 8-bit non-linear output paths, so adding opaque alpha to `gray-*-1.8.png` changed stored grayscale samples instead of preserving the encoded sRGB bytes expected by libpng's simplified API.
- Tests added: `safe/tests/read-transforms/simplified_read_driver.c` now compares `PNG_FORMAT_GRAY` and `PNG_FORMAT_GA` reads of `original/contrib/testpngs/gray-2-1.8.png` and asserts the grayscale byte is preserved while alpha is filled with `255`. This reproduces the pre-fix `gray-2-1.8.png` mismatch reported by `safe/tools/run-write-tests.sh pngstest-1.8`.
- Fixes applied: `safe/src/simplified_runtime.rs` now treats 8-bit simplified non-linear sources as sRGB-encoded samples, keeps 16-bit gamma handling intact, and allows the 8-bit direct read path to preserve samples when adding opaque alpha. `safe/tools/check-read-transforms.sh` was updated to pass the new gamma grayscale fixture to the existing read-transform smoke driver.
- Refreshed validator artifacts: `validator/artifacts/libpng-safe-source-api/`.

Validation commands:

```bash
cargo fmt --check
cargo test
safe/tools/check-exports.sh
safe/tools/check-headers.sh
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
safe/tools/run-read-tests.sh
safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha pngstest-large-stride
safe/tools/run-upstream-tests.sh
safe/tools/check-link-compat.sh
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -S -sa
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root "$PWD/artifacts/libpng-safe-source-api" --mode original --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides --library libpng --record-casts
```

Validation results:

- `cargo fmt --check`: exit code `0`.
- `cargo test`: exit code `0`.
- `safe/tools/check-exports.sh`: exit code `0`.
- `safe/tools/check-headers.sh`: exit code `0`.
- `safe/tools/check-core-smoke.sh`: exit code `0`.
- `safe/tools/check-read-core.sh`: exit code `0`.
- `safe/tools/check-read-transforms.sh`: exit code `0`.
- `safe/tools/run-read-tests.sh`: exit code `0`.
- `safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha pngstest-large-stride`: exit code `0`; the reported `gray-2-1.8.png`, `gray-4-1.8.png`, and `gray-8-1.8.png` opaque component mismatches are fixed.
- `safe/tools/run-upstream-tests.sh`: exit code `0`; `pngvalid-standard` passed, covering the upstream grayscale read validation matrix.
- `safe/tools/check-link-compat.sh`: exit code `0`.
- Source package rebuild: `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -S -sa` exit code `0`.
- Binary package rebuild: `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b` exit code `0`.
- `safe/tools/check-package-artifacts.sh`: exit code `0`; package artifacts match the current safe packaging tree and source snapshot.
- Full validator rerun: exit code `1`, with 77/105 passed, 28 failed, and 105 casts recorded.
- Source cases in the rerun: 5/5 passed. `chunk-metadata-inspection` and `read-write-c-api-smoke` both have `status: passed` and `exit_code: 0` in `validator/artifacts/libpng-safe-source-api/results/libpng/`.

Remaining later-phase failures:

- The 28 failed cases in `validator/artifacts/libpng-safe-source-api/results/libpng/` are the same deferred usage failures listed in the initial report: 26 Netpbm usage failures and 2 pngquant usage failures.
- No validator bug exception is claimed for this phase.
