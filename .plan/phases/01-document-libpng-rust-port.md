# Phase Name

Document libpng Rust port

# Implement Phase ID

`impl-document-libpng-rust-port`

# Preexisting Inputs

Consume the existing repository, prepared evidence, and package artifacts in place. Do not reclone, refetch, retarget, or regenerate existing validator evidence unless an expected input is missing or internally inconsistent.

- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/original/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/Cargo.toml`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/Cargo.lock`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/src/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/build.rs`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/cshim/longjmp_bridge.c`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/cshim/read_phase_bridge.c`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/include/png.h`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/include/pngconf.h`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/include/pnglibconf.h`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/abi/exports.txt`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/abi/libpng.vers`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/abi/install-layout.txt`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/pkg/libpng.pc.in`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/pkg/libpng-config.in`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/pkg/source-snapshot-manifest.txt`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/tools/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/tools/build_support/build_support.rs`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/tests/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/tests/cve-regressions/coverage.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/debian/control`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/debian/rules`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/debian/changelog`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/debian/patches/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/UNSAFE.md`
- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/TODO`
- `/home/yans/safelibs/pipeline/ports/port-libpng/dependents.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/relevant_cves.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/all_cves.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/validator/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/validator-case-inventory.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/validator-report.md`
- `/home/yans/safelibs/pipeline/ports/port-libpng/validator/artifacts/libpng-safe-final/results/libpng/summary.json`
- `/home/yans/safelibs/pipeline/ports/port-libpng/validator-overrides/libpng/`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng16-16t64_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng-dev_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng-tools_1.6.43-5ubuntu0.5+safelibs1_amd64.deb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng16-16-udeb_1.6.43-5ubuntu0.5+safelibs1_amd64.udeb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng16-16t64-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng-tools-dbgsym_1.6.43-5ubuntu0.5+safelibs1_amd64.ddeb`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1.dsc`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1.tar.xz`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43.orig.tar.xz`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.buildinfo`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1_amd64.changes`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.buildinfo`
- `/home/yans/safelibs/pipeline/ports/port-libpng/libpng1.6_1.6.43-5ubuntu0.5+safelibs1_source.changes`

# New Outputs

- `/home/yans/safelibs/pipeline/ports/port-libpng/safe/PORT.md`, created or updated in place.
- One git commit with message `docs: document libpng Rust port`.

# File Changes

- `safe/PORT.md`: create the authoritative port document with the six required sections in exact order. If this file appears before implementation begins, read it first and reconcile it rather than replacing still-accurate text wholesale.
- No code or packaging file changes are expected. If a real source or packaging bug is discovered while reconciling documentation, keep the fix minimal, cite it in `safe/PORT.md`, and include it in the same final commit.

# Implementation Details

## Workspace and artifact handling

1. Run `git status --short`.
2. Note existing untracked build/package outputs such as `safe/target`, `safe/debian/cargo-home`, `safe/debian/tmp`, `safe/debian/libpng*/`, `safe/debian/upstream-source-root`, root `.deb`, `.ddeb`, `.udeb`, `.dsc`, `.tar.xz`, `.buildinfo`, and `.changes` files. Treat them as existing build/package artifacts, not source edits.
3. Check whether `safe/PORT.md` exists. If it exists, preserve accurate prose and update only what current evidence requires.
4. Do not clone, pull, retarget, or query remote state for the existing `validator/` checkout. Validator evidence must be consumed from the checked-out commit and prepared artifacts.

## Architecture evidence

Gather and document the current port architecture from the existing tree:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
cargo metadata --locked --format-version 1 --no-deps --manifest-path safe/Cargo.toml
cargo tree --locked -e normal,build --manifest-path safe/Cargo.toml
```

Confirm and document:

- `safe/Cargo.toml` defines the `libpng-safe` package version `1.6.43`, edition `2024`, with library target `png16` and crate types `cdylib` and `staticlib`.
- There is no feature table, and the direct dependencies are `flate2 = "1"`, `libc = "0.2"`, `png = "0.18.1"`, plus build dependency `cc = "1.2"`.
- Public ABI files are `safe/include/png.h`, `safe/include/pngconf.h`, `safe/include/pnglibconf.h`, `safe/abi/exports.txt`, and `safe/abi/libpng.vers`.
- `safe/abi/exports.txt` contains 246 intended `png_*` exports, and `safe/tools/check-exports.sh` verifies the staged shared object against that baseline and `PNG16_0` symbol versioning.
- `safe/src/lib.rs` wires the Rust modules. Preserve these exact module responsibility mappings in the document:
  - Public C ABI shims are in `safe/src/common.rs`, `safe/src/memory.rs`, `safe/src/error.rs`, `safe/src/io.rs`, `safe/src/get.rs`, `safe/src/set.rs`, `safe/src/read_transform.rs`, `safe/src/read_progressive.rs`, `safe/src/compat_exports.rs`, `safe/src/write.rs`, `safe/src/write_util.rs`, `safe/src/write_transform.rs`, `safe/src/colorspace.rs`, `safe/src/interlace.rs`, and `safe/src/simplified.rs`.
  - Internal state is centered in `safe/src/state.rs`, with `PngStructState` and `PngInfoState` registries keyed by exported opaque `png_struct` and `png_info` pointer values.
  - Read-side parsing and transitions live mainly in `safe/src/read.rs`, `safe/src/read_util.rs`, `safe/src/chunks.rs`, `safe/src/read_progressive.rs`, and `safe/src/bridge_ffi.rs`.
  - Write-side behavior lives mainly in `safe/src/write.rs`, `safe/src/write_runtime.rs`, `safe/src/write_util.rs`, `safe/src/write_transform.rs`, and `safe/src/bridge_ffi.rs`, with Rust `png` and `flate2` used in the write runtime.
  - Simplified `png_image_*` APIs are implemented through `safe/src/simplified.rs`, `safe/src/simplified_runtime.rs`, and bridge helpers in `safe/src/bridge_ffi.rs`.
  - Compatibility exports, transforms, colorspace, interlace, zlib, I/O, memory, and error handling should be described by their concrete source files rather than as unnamed subsystems.
- `safe/src/state.rs` owns `PngStructState` and `PngInfoState` registries keyed by exported opaque `png_struct` and `png_info` pointer values.
- `safe/src/common.rs:121` and `safe/src/common.rs:137` define the ABI panic-containment guards.
- `safe/cshim/longjmp_bridge.c` and `safe/cshim/read_phase_bridge.c` remain checked-in C boundary documentation/glue.
- `safe/tools/build_support/build_support.rs` must be cited for the note that the old hidden support runtime is no longer synthesized or compiled.
- `safe/build.rs`, `safe/tools/stage-install-tree.sh`, `safe/pkg/libpng.pc.in`, `safe/pkg/libpng-config.in`, `safe/debian/control`, and `safe/debian/rules` define build, staging, link, install-surface, and Debian packaging behavior.
- `safe/debian/control` declares the exact build dependencies `debhelper-compat (= 13), cargo, dpkg-dev (>= 1.22.5), gcc, mawk, python3, rustc, zlib1g-dev`; preserve that list exactly when documenting Debian build inputs.
- `safe/debian/rules` builds the Rust crate, stages headers/libs/pkg-config/config scripts, builds `pngfix` and `png-fix-itxt`, runs upstream/check-link tests, and packages `libpng16-16t64`, `libpng-dev`, `libpng-tools`, and `libpng16-16-udeb`.
- Include a compact directory map for `safe/src`, `safe/include`, `safe/abi`, `safe/pkg`, `safe/tools`, `safe/tests`, `safe/cshim`, `safe/contrib`, and `safe/debian`.

## Unsafe inventory

Run both the tracked-source inventory and the required full-tree search:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
git ls-files safe/src safe/build.rs safe/cshim safe/tools/build_support/build_support.rs \
  | xargs rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl'
grep -RIn '\bunsafe\b' safe
```

Use tracked Rust source as the authoritative implementation inventory. Reconcile full-tree matches by explaining that generated Cargo caches, build outputs, Debian patches, upstream prose, comments, or binary artifacts may appear in `grep -RIn` but are not all live tracked Rust implementation.

Enumerate every tracked Rust `unsafe extern "C"` block, `#[unsafe(no_mangle)] pub unsafe extern "C" fn`, non-exported `unsafe fn`, unsafe block, and `unsafe impl` by file and line, with concise justification. Group the inventory by:

- Public libpng ABI entry points and panic guards.
- Internal `bridge_` and `png_safe_` ABI shims.
- Raw pointer reads/writes and C string/slice conversion.
- Callback invocation into application-provided C functions.
- Allocator integration and opaque handle lifetime.
- `setjmp`/`longjmp` compatibility.
- zlib FFI.
- stdio/file/time/libc helpers.
- `unsafe impl Send` for registry state in `safe/src/state.rs:75` and `safe/src/state.rs:227`.

Call out unsafe that is not strictly required by the public C ABI boundary, including direct zlib FFI, internal Rust-to-Rust extern shims, libc stdio/file helpers used by simplified APIs, and `unsafe impl Send`.

## Remaining FFI beyond the original ABI/API boundary

Document remaining system FFI and replacement possibilities:

- zlib symbols from `safe/src/zlib.rs:31`: `zlibVersion`, `inflateInit_`, `inflate`, `inflateEnd`.
- libc allocation and longjmp symbols from `safe/src/memory.rs` and `safe/src/error.rs`: `_setjmp`, `longjmp`, `malloc`, `calloc`, `free`.
- stdio/file symbols from `safe/src/io.rs`, `safe/src/bridge_ffi.rs`, and `safe/src/simplified_runtime.rs`: `fopen`, `fclose`, `ftell`, `fseek`, `fread`, `fwrite`, `fflush`.
- string/time/math helpers from `safe/src/bridge_ffi.rs` and `safe/src/common.rs`: `strlen`, `atof`, `memcmp`, `gmtime_r`.
- Runtime link evidence from:

```bash
readelf -d safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0
```

The current expectation is that `NEEDED` includes `libz.so.1`, `libm.so.6`, `libgcc_s.so.1`, `libc.so.6`, and `ld-linux-x86-64.so.2`, with SONAME `libpng16.so.16`. Explain that `safe/build.rs` and `safe/tools/stage-install-tree.sh` pass `-lz -lm -ldl -lpthread -lrt -lutil -lgcc_s`, and document which remain after linker `--as-needed` behavior.

For each FFI surface, state why it remains and what could replace it with safer Rust later:

- Replace direct zlib inflate FFI with a pure Rust or `flate2` inflate path if compatibility and performance are acceptable.
- Replace libc stdio/file helpers in simplified APIs with Rust-owned file reads where the libpng ABI permits ownership and callback semantics to remain compatible.
- Replace raw callback registration/calls with safer wrapper types that validate callback pointers, lifetimes, and user data at the boundary.
- Reduce or remove `setjmp`/`longjmp` handling only if libpng ABI compatibility is relaxed, since existing callers rely on that error-control model.
- Collapse Rust-to-Rust internal `extern "C"` shims back to ordinary Rust calls where they are not needed for exported ABI or C bridge interop.
- Revisit `unsafe impl Send` for registry state if a more constrained synchronization or ownership model can preserve ABI behavior.

## Remaining issues and validation evidence

Use existing evidence in place:

- Verify that `git -C validator rev-parse HEAD`, `validator-case-inventory.json`, and `validator-report.md` all name validator commit `cc99047419226144eec3c1ab87873052bd9abedc`.
- Cite the existing final local-override validator result: 105/105 libpng validator cases passed, 0 failed, no validator bug exceptions.
- Use `validator/artifacts/libpng-safe-final/results/libpng/summary.json`, `validator-report.md`, and `validator-case-inventory.json`; do not regenerate inventories or retarget the checkout.
- Use `safe/tools/` and `safe/tests/` to list local build/test harnesses.
- Find live TODO/FIXME markers with:

```bash
rg -n "TODO|FIXME|XXX" safe/src safe/tests safe/tools safe/debian --glob '!safe/debian/cargo-home/**' --glob '!safe/debian/tmp/**' --glob '!safe/debian/libpng*/**'
```

Distinguish upstream TODO/prose and Debian patch history from live Rust-port issues. Treat `safe/TODO` as upstream libpng TODO content, not automatically as Rust-port-specific defects.

Section 4 must explicitly address:

- Performance regressions versus `original/`, using current evidence such as `timepng` smoke coverage if present. If there is no comprehensive quantitative benchmark comparing throughput or latency against original C libpng, say so.
- Bit-for-bit behavioral equivalence, using exact `cmp`, `memcmp`, `pngstest-*`, baseline, and validator evidence where available. If existing artifacts do not prove exhaustive byte equivalence across all read/write behavior, say so and name the covered exact-comparison areas separately from uncovered areas.
- CVE coverage by comparing `relevant_cves.json` with `safe/tests/cve-regressions/coverage.json`.
- Dependent coverage from `dependents.json`, `validator-case-inventory.json`, `validator-report.md`, and `safe/tools/run-dependent-regressions.sh`.
- Distinguish actual validator execution from inventory-only entries. Current explicit validator execution covers Netpbm and pngquant usage cases, and `safe/tests/dependents/` contains targeted C regressions for API patterns.
- Describe GIMP, LibreOffice Draw, Scribus, WebKitGTK, GDK Pixbuf, Cairo, SDL2_image, feh, XSane, and the R `png` package as dependent inventory rather than direct end-to-end validation unless an existing artifact proves direct coverage.
- Packaging caveats from the final validator report, including local override proof-tooling limitations if present.

Use these searches to ground the evidence:

```bash
rg -n "timepng|performance|bench|perf" safe/tools safe/tests validator-report.md original safe/TODO
rg -n "cmp -s|memcmp|exact|bit-for-bit|byte-for-byte|pngstest|baseline" safe/tools safe/tests validator-report.md
```

## Dependencies

Section 5 must list direct dependencies from `safe/Cargo.toml` with manifest requirements and resolved versions from `cargo tree`:

- `flate2 = "1"`, resolved to `1.1.9`: zlib/deflate encoding support in write runtime.
- `libc = "0.2"`, resolved to `0.2.183`: C ABI types and direct libc calls.
- `png = "0.18.1"`, resolved to `0.18.1`: safe Rust PNG encoder/decoder building blocks for simplified and write paths.
- Build dependency `cc = "1.2"`, resolved to `1.2.58`: compiler discovery for staged shared-library linking in `build.rs`.

Include transitive dependencies only as supporting context. Also document system/build dependencies: zlib, libm, libc, libgcc_s, dynamic loader, `dl`, `pthread`, `rt`, `util`, Cargo, rustc, gcc/cc, dpkg-dev, debhelper, mawk, python3, `pkg-config`, `nm`, `objdump`, `readelf`, and optional `cargo geiger`.

State that `cbindgen` and `bindgen` are not present in `safe/Cargo.toml` or `safe/build.rs`.

Identify Rust dependencies that are unsafe-heavy or do not declare `#![forbid(unsafe_code)]`, using `cargo geiger` when available, or manual registry/source inspection when it is not. Prefer `safe/debian/cargo-home/registry/src` when present. If `cargo geiger` is unavailable, section 6 must say so and section 5 must record the manual inspection command.

## Required document sections

Write `safe/PORT.md` with these exact headings in this exact order:

```markdown
## 1. High-level architecture
## 2. Where the unsafe Rust lives
## 3. Remaining unsafe FFI beyond the original ABI/API boundary
## 4. Remaining issues
## 5. Dependencies and other libraries used
## 6. How this document was produced
```

Keep the document self-contained. Prefer tables for unsafe and FFI inventories. Include refreshable file:line references and command names in section 6.

## Implementation-side verification before yielding

Run or record as unavailable with a reason:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
cargo metadata --locked --format-version 1 --no-deps --manifest-path safe/Cargo.toml
cargo tree --locked -e normal,build --manifest-path safe/Cargo.toml
cargo geiger --manifest-path safe/Cargo.toml || true
grep -RIn '\bunsafe\b' safe >/tmp/libpng-safe-unsafe-full.txt
git ls-files safe/src safe/build.rs safe/cshim safe/tools/build_support/build_support.rs | xargs rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl' >/tmp/libpng-safe-unsafe-tracked.txt
cargo build --locked --release --manifest-path safe/Cargo.toml
safe/tools/stage-install-tree.sh
safe/tools/check-exports.sh
safe/tools/check-headers.sh
safe/tools/check-link-compat.sh
safe/tools/check-install-surface.sh
safe/tools/check-build-layout.sh
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
safe/tools/run-read-tests.sh
safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-large-stride pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha
safe/tools/run-upstream-tests.sh
safe/tools/check-examples-and-tools.sh
safe/tools/run-cve-regressions.sh --mode all
safe/tools/run-dependent-regressions.sh
jq -e '.passed == .cases and .failed == 0' validator/artifacts/libpng-safe-final/results/libpng/summary.json
validator_commit="$(git -C validator rev-parse HEAD)"
test "$validator_commit" = "cc99047419226144eec3c1ab87873052bd9abedc"
test "$validator_commit" = "$(jq -r '.validator_commit' validator-case-inventory.json)"
rg -n "timepng|performance|bench|perf|cmp -s|memcmp|exact|bit-for-bit|byte-for-byte|pngstest|baseline" safe/tools safe/tests validator-report.md
git diff --check -- safe/PORT.md
git log -1 --oneline --name-only -- safe/PORT.md
```

Before yielding, verify cited paths exist, cited symbols are findable with `rg`, dependency claims match `safe/Cargo.toml`, and the unsafe inventory reconciles the tracked-source search with the full-tree search.

# Verification Phases

## `check-port-doc-structure-and-grounding`

- Type: `check`
- Fixed `bounce_target`: `impl-document-libpng-rust-port`
- Purpose: Verify that `safe/PORT.md` exists, has the required sections in order, preserves or creates accurate prose, and grounds file paths/symbols in existing files.
- Commands:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
test -f safe/PORT.md
python3 - <<'PY'
from pathlib import Path
text = Path("safe/PORT.md").read_text()
sections = [
    "## 1. High-level architecture",
    "## 2. Where the unsafe Rust lives",
    "## 3. Remaining unsafe FFI beyond the original ABI/API boundary",
    "## 4. Remaining issues",
    "## 5. Dependencies and other libraries used",
    "## 6. How this document was produced",
]
pos = -1
for section in sections:
    next_pos = text.find(section)
    if next_pos <= pos:
        raise SystemExit(f"missing or out-of-order section: {section}")
    pos = next_pos
PY
python3 - <<'PY'
import os
import re
from pathlib import Path
text = Path("safe/PORT.md").read_text()
paths = sorted(set(re.findall(r'(?<![A-Za-z0-9_./-])((?:safe|original|validator|validator-overrides)[/][A-Za-z0-9_./+-]+)', text)))
missing = [p for p in paths if not os.path.exists(p)]
if missing:
    raise SystemExit("missing cited paths:\n" + "\n".join(missing))
PY
rg -n "libpng-safe|png16|cdylib|staticlib|abi/exports.txt|abi/libpng.vers|debian/rules|validator-report.md" safe/PORT.md
rg -n "png_create_read_struct|png_read_info|png_write_info|png_image_begin_read_from_memory|PngStructState|PngInfoState|abi_guard" safe/PORT.md
rg -n "performance|timepng|bit-for-bit|byte-for-byte|equivalent|equivalence" safe/PORT.md
```

## `check-port-doc-evidence-and-commands`

- Type: `check`
- Fixed `bounce_target`: `impl-document-libpng-rust-port`
- Purpose: Verify that the documentation can be refreshed from the stated commands and that unsafe, FFI, dependency, export, package, CVE, dependent, and test claims match the current workspace.
- Commands:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
cargo metadata --locked --format-version 1 --no-deps --manifest-path safe/Cargo.toml
cargo tree --locked -e normal,build --manifest-path safe/Cargo.toml
cargo geiger --manifest-path safe/Cargo.toml || true
grep -RIn '\bunsafe\b' safe >/tmp/libpng-safe-unsafe-full.txt
git ls-files safe/src safe/build.rs safe/cshim safe/tools/build_support/build_support.rs \
  | xargs rg -n '\bunsafe\b|unsafe extern "C"|#\[unsafe|unsafe impl' \
  >/tmp/libpng-safe-unsafe-tracked.txt
git ls-files safe/src \
  | xargs rg -n 'extern "C"|libc::|zlibVersion|inflateInit_|inflateEnd|inflate\(|fopen|fclose|ftell|fseek|fread|fwrite|fflush|strlen|atof|memcmp|gmtime_r|_setjmp|longjmp|malloc|calloc|free' \
  >/tmp/libpng-safe-ffi-tracked.txt
cargo build --locked --release --manifest-path safe/Cargo.toml
safe/tools/stage-install-tree.sh
safe/tools/check-exports.sh
safe/tools/check-headers.sh
safe/tools/check-link-compat.sh
safe/tools/check-install-surface.sh
safe/tools/check-build-layout.sh
readelf -d safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0 | rg 'NEEDED|SONAME'
nm -D --defined-only safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0 | rg 'PNG16_0| png_' | sed -n '1,20p'
objdump -T safe/target/release/abi-stage/usr/lib/x86_64-linux-gnu/libpng16.so.16.43.0 | sed -n '1,80p'
validator_commit="$(git -C validator rev-parse HEAD)"
test "$validator_commit" = "cc99047419226144eec3c1ab87873052bd9abedc"
test "$validator_commit" = "$(jq -r '.validator_commit' validator-case-inventory.json)"
grep -F "Validator commit: \`$validator_commit\`." validator-report.md
jq -e '.cases == 105 and .source_cases == 5 and .usage_cases == 100 and .original_mode_override_supported == true and .proof_rejects_original_override == true' validator-case-inventory.json
jq -e '.passed == .cases and .failed == 0 and .cases == 105' validator/artifacts/libpng-safe-final/results/libpng/summary.json
jq -e '.selected_cve_coverage | keys | length > 0' safe/tests/cve-regressions/coverage.json
python3 - <<'PY'
import json
from pathlib import Path
relevant = json.loads(Path("relevant_cves.json").read_text())
selected = {item["cve_id"] for bucket in ("high_priority", "secondary") for item in relevant[bucket]}
covered = set(json.loads(Path("safe/tests/cve-regressions/coverage.json").read_text())["selected_cve_coverage"])
if selected != covered:
    raise SystemExit(f"CVE coverage mismatch: missing={sorted(selected - covered)} extra={sorted(covered - selected)}")
PY
jq -e '.dependents | length == 12' dependents.json
safe/tools/check-core-smoke.sh
safe/tools/check-read-core.sh
safe/tools/check-read-transforms.sh
safe/tools/run-read-tests.sh
safe/tools/run-write-tests.sh pngstest-1.8 pngstest-1.8-alpha pngstest-large-stride pngstest-linear pngstest-linear-alpha pngstest-none pngstest-none-alpha pngstest-sRGB pngstest-sRGB-alpha
safe/tools/run-upstream-tests.sh
safe/tools/check-examples-and-tools.sh
safe/tools/run-cve-regressions.sh --mode all
safe/tools/run-dependent-regressions.sh
rg -n "timepng|cmp -s|memcmp|pngstest|bit|exact|baseline" safe/tools safe/tests validator-report.md
```

Manual review checks:

- Every unsafe item in `/tmp/libpng-safe-unsafe-tracked.txt` is represented in section 2 or in an explicitly documented generated-artifact exclusion note.
- Every direct dependency in `safe/Cargo.toml` appears in section 5.
- Section 5 identifies unsafe-heavy or non-`#![forbid(unsafe_code)]` Rust dependencies with an acceptability rationale grounded in their role in the port.
- Section 4 contains distinct Remaining issues entries for performance regressions versus original C libpng and for behavior not yet proven bit-for-bit equivalent.
- Section 4 backs those entries with concrete evidence or an explicit statement that no current artifact proves a regression or exhaustive equivalence.

## `check-port-doc-final-git-state`

- Type: `check`
- Fixed `bounce_target`: `impl-document-libpng-rust-port`
- Purpose: Verify that the documentation was committed in one git commit and that no unintended tracked files were changed.
- Commands:

```bash
cd /home/yans/safelibs/pipeline/ports/port-libpng
git log -1 --oneline --name-only -- safe/PORT.md
git show --stat --oneline HEAD -- safe/PORT.md
git diff --check HEAD~1..HEAD -- safe/PORT.md
git status --short
```

If `git status --short` shows preexisting untracked package/build outputs, do not fail solely for those. Fail for uncommitted tracked changes or untracked documentation/evidence files created by the workflow outside `safe/PORT.md`.

# Success Criteria

- `safe/PORT.md` exists and contains the six required sections in order.
- The document is grounded in current source, packaging, tests, dependency metadata, validator evidence, CVE/dependent data, and existing package artifacts.
- The consume-existing-artifacts contract is preserved: existing validator checkout/artifacts, CVE/dependent files, and package outputs are consumed in place rather than rediscovered or regenerated.
- The unsafe inventory reconciles the full-tree `grep -RIn '\bunsafe\b' safe` output with tracked Rust source.
- Remaining FFI, system links, direct Rust dependencies, unsafe-heavy/non-forbid dependencies, CVE coverage, dependent coverage, performance caveats, and bit-for-bit equivalence caveats are all explicitly documented.
- Validator evidence remains pinned to `cc99047419226144eec3c1ab87873052bd9abedc` and the final existing summary remains 105/105 passing with 0 failures.
- All verifier phases pass or bounce only to `impl-document-libpng-rust-port`.
- The final repository state has no uncommitted tracked changes introduced by the documentation workflow.

# Git Commit Requirement

The implementer must commit work to git before yielding. The required commit message is `docs: document libpng Rust port`, and the commit must include `safe/PORT.md` plus only any genuinely necessary incidental fixes documented in that file.
