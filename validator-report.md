# Validator Report: libpng-safe Initial Baseline

## Summary

- Phase: `impl-validator-baseline`.
- Validator checkout: `validator/`.
- Validator commit: `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`.
- Mode: validator `original` mode with local safe `.deb` overrides from `validator-overrides/libpng/`.
- Initial artifact root: `validator/artifacts/libpng-safe-initial/`.
- Initial validator exit code: `0`.
- Initial result: 135/135 passed, 0 failed, 135 casts recorded.
- Inventory match: 135 total cases, 5 source cases, 130 usage cases, matching `validator-case-inventory.json`.
- Validator source changes: none; the validator checkout is clean apart from ignored generated artifacts.

## Checks Executed

The required validator and package checks completed successfully:

```bash
git -C validator pull --ff-only
git -C validator rev-parse HEAD
cd validator && make unit
cd validator && make check-testcases
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
safe/tools/check-package-artifacts.sh
cd validator && bash test.sh --config repositories.yml --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-initial" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
```

`safe/tools/check-package-artifacts.sh` initially detected that the safe source
snapshot omitted tracked `safe/PORT.md`. The package snapshot manifest was
updated to include `PORT.md`, then source package artifacts were refreshed with:

```bash
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -S -sa
safe/tools/check-package-artifacts.sh
```

The final package gate confirmed that package artifacts passed, the source
package artifacts match the current safe packaging tree, the safe source
snapshot tar matches the current tracked safe tree, and `libpng-dev` examples
are preserved.

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
| `e4cbab99737e5e2bc4b7a402e5f292db0ad3342b40029310be5745cc0ac8cb74` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.dsc` |
| `e1771f5eb16560c498ea7327663ff07cb1768a1d84fc461048bf9ad04d1545c8` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz` |
| `ec6a4c651d068f83c4c41527f737ab5bab70a78ed01ddf3c3632ca720510a9bc` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1.tar.xz` |
| `245573d767b5374b12e0d261b69d38c48236b15581c5cf3de8b46caa494e4ba5` | `libpng1.6_1.6.43.orig.tar.xz` |
| `392eb0ea6445677ec3954013362ba1bb594f09ed40b14190098b318c487cc1a9` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `9006ea70acb6ac634dbe640739600f5faea2fdad6262a5666afb46d60561b6cd` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |
| `c836307b908defdd17c1e944ec024a72806682ff75e5537559ba91668f52366f` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.buildinfo` |
| `dfd534231d2cf5f42a449a9501fbc2018ea0d55b335ec667238bf33dd2da7589` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.changes` |

Local validator override SHA-256 values:

| SHA-256 | Override artifact |
| --- | --- |
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |

## Initial Validator Results

- Summary JSON: `validator/artifacts/libpng-safe-initial/results/libpng/summary.json`.
- Validator exit code file: `validator/artifacts/libpng-safe-initial/validator.exit-code`.
- Result JSON files: 135 testcase files plus `summary.json`.
- Cast files: 135, one per testcase.
- Testcase logs: 135, one per testcase, plus `docker-build.log`.
- Source cases: 5/5 passed.
- Usage cases: 130/130 passed.
- Artifact consistency check: every testcase ID from `validator-case-inventory.json` has a matching result JSON, cast, and testcase log; every per-case result has `status: passed`, `exit_code: 0`, `mode: original`, and `override_debs_installed: true`.

Initial summary:

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

## Initial Failure Classification

No testcase failed in the initial full validator run.

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

The explicit mapping for future failed IDs is:
`chunk-metadata-inspection` and `read-write-c-api-smoke` are Source/API;
`malformed-png-rejection`, `palette-fixture-handling`, and
`pngfix-fixture-handling` are CLI/source fixtures; IDs beginning
`usage-netpbm-` are Netpbm usage; IDs beginning `usage-pngquant-` are pngquant
usage; every other failing ID is Other/catch-all.

## Inventory And Proof Notes

`validator-case-inventory.json` was recomputed from the validator libpng
testcase files after the validator update. The inventory records 5 source
cases, 130 usage cases, and validator commit
`5d908be26e33f071e119ffe1a52e3149f1e5ec4e`.

`validator-case-inventory.json` records `original_mode_override_supported:
true` because `test.sh --mode original --override-deb-root ...` installs the
local override packages. It also records `proof_rejects_original_override:
true`; at this validator commit, proof generation still rejects original-mode
result JSON when `override_debs_installed` is `true`. Proof/site targets were
not part of this baseline phase.
