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

## Source/API Validator Fix Phase

- Phase: `impl-source-api-validator-failures`.
- Baseline source/API status: no source/API failures in the current run.
  `chunk-metadata-inspection` and `read-write-c-api-smoke` both passed in
  `validator/artifacts/libpng-safe-initial/results/libpng/`.
- Local source/API reproducers added: none. The requested validator cases
  already passed against the staged headers and override packages.
- Safe source files changed: none.
- Package rebuild: completed with
  `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`; the rebuilt
  runtime, dev, and tools debs were refreshed into
  `validator-overrides/libpng/`. SHA-256 values remained unchanged from the
  initial baseline.
- Package artifact gate: `safe/tools/check-package-artifacts.sh` passed.
- Local verifier battery passed:
  `cargo build --locked --release --manifest-path safe/Cargo.toml`,
  `safe/tools/check-exports.sh`, `safe/tools/check-headers.sh`,
  `safe/tools/check-link-compat.sh`, `safe/tools/check-install-surface.sh`,
  `safe/tools/check-build-layout.sh`, `safe/tools/check-core-smoke.sh`,
  `safe/tools/check-read-core.sh`, `safe/tools/check-read-transforms.sh`,
  `safe/tools/run-read-tests.sh`, the required `safe/tools/run-write-tests.sh`
  `pngstest-*` matrix, `safe/tools/run-upstream-tests.sh`,
  `safe/tools/check-examples-and-tools.sh`,
  `safe/tools/run-cve-regressions.sh --mode all`, and
  `safe/tools/run-dependent-regressions.sh`.
- Fresh full validator artifact root:
  `validator/artifacts/libpng-safe-source-api/`.
- Source/API phase validator exit code:
  `validator/artifacts/libpng-safe-source-api/validator.exit-code` contains
  `0`.

Source/API phase summary:

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

The specific source/API validator cases remain green in the fresh run:

| Testcase ID | Status | Exit code |
| --- | --- | --- |
| `chunk-metadata-inspection` | passed | 0 |
| `read-write-c-api-smoke` | passed | 0 |

Remaining failure classification after this phase:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

## CLI/Source Validator Fix Phase

- Phase: `impl-cli-source-validator-failures`.
- Baseline CLI/source fixture status: no CLI/source failures in the current
  source/API run. `malformed-png-rejection`, `palette-fixture-handling`, and
  `pngfix-fixture-handling` all passed in
  `validator/artifacts/libpng-safe-source-api/results/libpng/`.
- Local regression updates added: none. The requested CLI/source validator
  cases already passed against the staged safe build and override packages.
- Safe source, tool, and packaging files changed: none.
- Local verifier battery passed:
  `safe/tests/upstream/pngfix.sh`,
  `safe/tools/check-examples-and-tools.sh`, and a targeted temporary pngfix
  fixture check covering non-PNG rejection, `basn3p08.png` output creation,
  and `basn2c08.png` readable PNG output using fixtures from
  `original/contrib/pngsuite/`.
- Package rebuild: completed with
  `cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b`; the rebuilt
  runtime, dev, and tools debs were refreshed into
  `validator-overrides/libpng/`. SHA-256 values remained unchanged.
- Package artifact gate: `safe/tools/check-package-artifacts.sh` passed.
- Fresh full validator artifact root:
  `validator/artifacts/libpng-safe-cli-source/`.
- CLI/source phase validator exit code:
  `validator/artifacts/libpng-safe-cli-source/validator.exit-code` contains
  `0`.

Commands executed for this phase:

```bash
safe/tests/upstream/pngfix.sh
safe/tools/check-examples-and-tools.sh
bash -lc 'set -euo pipefail
source safe/tests/upstream/common.sh
build_dir="$(mktemp -d)"
trap "rm -rf \"$build_dir\"" EXIT
build_pngfix_consumer "$build_dir"
printf "not png\n" >"$build_dir/bad.png"
if "$build_dir/pngfix" --out="$build_dir/bad-out.png" "$build_dir/bad.png" >"$build_dir/bad.log" 2>&1; then
  cat "$build_dir/bad.log"
  printf "pngfix unexpectedly accepted non-PNG input\n" >&2
  exit 1
fi
if [[ -s "$build_dir/bad-out.png" ]] && file "$build_dir/bad-out.png" | grep -q "PNG image data"; then
  printf "pngfix produced a valid PNG for malformed input\n" >&2
  exit 1
fi
"$build_dir/pngfix" --out="$build_dir/basn3p08-out.png" original/contrib/pngsuite/basn3p08.png >/dev/null
[[ -s "$build_dir/basn3p08-out.png" ]]
"$build_dir/pngfix" --out="$build_dir/basn2c08-out.png" original/contrib/pngsuite/basn2c08.png >/dev/null
file "$build_dir/basn2c08-out.png" | grep -q "PNG image data"
printf "targeted local pngfix fixture checks passed\n"'
cd safe && ./tools/dpkg-buildpackage-wrapper.sh -us -uc -b
cp -f libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb \
  libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb \
  libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb \
  validator-overrides/libpng/
safe/tools/check-package-artifacts.sh
cd validator && bash test.sh --config repositories.yml --tests-root tests \
  --artifact-root "$PWD/artifacts/libpng-safe-cli-source" \
  --mode original \
  --override-deb-root /home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides \
  --library libpng \
  --record-casts
```

CLI/source phase package artifact SHA-256 values:

| SHA-256 | Artifact |
| --- | --- |
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `1c567d67fbc99e6a32015d434895edb5bd2bbcdeb810a749d80e5f4745dcce4b` | `libpng-tools-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `0b0697d920eba71496e56b3be1c175be60b7df2835ea7f5f3de7ef933db82b6e` | `libpng16-16t64-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb` |
| `f07558cabbc0cf6d369cb695d040dfdb207326d0ac5b0be1eabb7575e34fdc97` | `libpng16-16-udeb_1.6.43-5ubuntu0.5+safelibs1_amd64.udeb` |
| `392eb0ea6445677ec3954013362ba1bb594f09ed40b14190098b318c487cc1a9` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo` |
| `9006ea70acb6ac634dbe640739600f5faea2fdad6262a5666afb46d60561b6cd` | `libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes` |

CLI/source phase override SHA-256 values:

| SHA-256 | Override artifact |
| --- | --- |
| `e4284ee097a820e934d154675179140d49417276f80fed273b223ce16ab9c8d8` | `validator-overrides/libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `410e64ccf940aa321584d670326876a3a61406003d44fa30f8c40e94fa1a3886` | `validator-overrides/libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |
| `9685e238a815c5eac1dcb87ef55972072aac07f5f7ccd00e53a03968ac28abf7` | `validator-overrides/libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb` |

CLI/source phase summary:

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

The specific CLI/source validator cases remain green in the fresh run:

| Testcase ID | Status | Exit code |
| --- | --- | --- |
| `malformed-png-rejection` | passed | 0 |
| `palette-fixture-handling` | passed | 0 |
| `pngfix-fixture-handling` | passed | 0 |

Remaining failure classification after this phase:

| Classification | Failing testcase IDs |
| --- | --- |
| Source/API | none |
| CLI/source fixtures | none |
| Netpbm usage | none |
| pngquant usage | none |
| Other/catch-all | none |

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
