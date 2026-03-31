# Unsafe Boundaries

This phase still ships a mixed Rust and upstream-C libpng. The hybrid baseline
is intentional and is defined by `safe/build.rs`, `safe/src/lib.rs`,
`safe/cshim/read_phase_bridge.c`, and the frozen ABI artifacts under
`safe/abi/`.

## Current Hybrid Baseline

- `safe/build.rs` still compiles the upstream core libpng C sources from
  `original/` into `libpng16_upstream.a`.
- The frozen public ABI currently contains 246 `png_*` exports
  (`safe/abi/exports.txt`).
- `UPSTREAM_RENAMES` in `safe/build.rs` renames 38 public symbols to
  `upstream_*` before the upstream C objects are linked.
- 37 of those 38 renamed symbols are re-owned by Rust.
- 1 renamed symbol, `png_read_row`, is re-owned by the active C shim in
  `safe/cshim/read_phase_bridge.c`.
- The remaining 208 public `png_*` exports are still upstream-C owned.

The compiled Rust crate root is deliberately small:

- Public ABI owners:
  `safe/src/common.rs`, `safe/src/read_transform.rs`,
  `safe/src/interlace.rs`, and `safe/src/colorspace.rs`
- Compiled support modules without additional public ABI ownership:
  `safe/src/chunks.rs`, `safe/src/read_progressive.rs`,
  `safe/src/read_util.rs`, `safe/src/write.rs`,
  `safe/src/write_transform.rs`, `safe/src/write_util.rs`,
  `safe/src/zlib.rs`, `safe/src/types.rs`, and `safe/src/abi_exports.rs`

## Rust-Owned Public Symbols

These symbol families are the Rust-owned part of the current public ABI:

- `safe/src/common.rs`:
  `png_sig_cmp`, `png_access_version_number`, `png_get_libpng_ver`,
  `png_get_header_ver`, `png_get_header_version`, `png_get_copyright`,
  `png_build_grayscale_palette`, `png_save_uint_32`, `png_save_uint_16`,
  `png_save_int_32`, `png_convert_to_rfc1123_buffer`,
  `png_convert_from_struct_tm`, and `png_convert_from_time_t`
- `safe/src/read_transform.rs`:
  `png_set_expand`, `png_set_expand_16`, `png_set_palette_to_rgb`,
  `png_set_tRNS_to_alpha`, `png_set_gray_to_rgb`, `png_set_scale_16`,
  `png_set_strip_16`, `png_set_quantize`, `png_set_shift`,
  `png_set_swap_alpha`, `png_set_invert_alpha`, `png_set_invert_mono`,
  and `png_set_bgr`
- `safe/src/interlace.rs`:
  `png_set_interlace_handling`
- `safe/src/colorspace.rs`:
  `png_set_rgb_to_gray`, `png_set_rgb_to_gray_fixed`,
  `png_set_background`, `png_set_background_fixed`,
  `png_set_alpha_mode`, `png_set_alpha_mode_fixed`,
  `png_set_cHRM_XYZ`, `png_set_cHRM_XYZ_fixed`,
  `png_get_cHRM_XYZ`, and `png_get_cHRM_XYZ_fixed`

These exports use `abi_guard!` or `abi_guard_no_png!` only for panic
containment. They abort on panic; they do not translate Rust panic into
`png_error` or `png_longjmp`.

## Upstream-Owned Public ABI

Everything not renamed in `safe/build.rs` still comes from the upstream C
objects. In practice that means the current shipped ABI still leaves these
areas upstream-owned:

- error handling and longjmp registration
- allocator hooks and struct allocation/destruction
- IO callback registration and progressive callback setup
- most metadata getters and setters
- sequential read core, progressive read core, and write core
- simplified `png_image_*` entry points
- the remaining parser, chunk, and transform helpers not listed above

The files below are reference implementations for those families, but they are
not wired into `safe/src/lib.rs` in this phase:

- `safe/src/error.rs`
- `safe/src/memory.rs`
- `safe/src/io.rs`
- `safe/src/get.rs`
- `safe/src/set.rs`
- `safe/src/simplified.rs`

## Active C Shim Boundary

`safe/cshim/read_phase_bridge.c` is the only compiled C shim in this phase.
It owns both the private-layout mirror boundary and the longjmp containment
needed by the Rust-owned read-transform and colorspace entry points.

It provides these active helper paths:

- private state snapshot and restore:
  `png_safe_read_core_get`, `png_safe_read_core_set`,
  `png_safe_info_core_get`, and `png_safe_info_core_set`
- longjmp-contained upstream calls:
  `png_safe_call_read_info`, `png_safe_call_read_update_info`,
  `png_safe_call_read_image`, `png_safe_call_read_end`,
  `png_safe_call_warning`, `png_safe_call_benign_error`,
  `png_safe_call_app_error`, `png_safe_call_error`,
  and `png_safe_call_set_quantize`
- public ABI symbol re-owned by the shim:
  `png_read_row`

The shim remains necessary because later phases have not yet removed:

- private `png_struct` and `png_info` field mirroring
- upstream fatal-error paths that still use `setjmp`/`longjmp`
- the packed-row padding hardening that currently lives inside the shimmed
  `png_read_row`

## Dormant Modules

These files remain in the tree but are not part of the active compiled unsafe
surface for this phase:

- `safe/src/read.rs`
- `safe/src/state.rs`
- `safe/cshim/longjmp_bridge.c`

They are intentionally dormant. Later phases may either delete them or revive
pieces of them as the remaining upstream-owned ABI families move into Rust.

## Remaining Compiled Unsafe

The active compiled `unsafe` is currently limited to boundary code that still
has to cross the C ABI:

- `extern "C"` entry points receiving foreign `png_struct*`, `png_info*`,
  buffers, and callback pointers
- raw pointer reads and writes inside the active Rust ABI owners
- FFI calls from Rust into `safe/cshim/read_phase_bridge.c`
- private-layout mirroring in the active C shim
- C-side `setjmp` containment around upstream longjmp-capable calls

## Target End-State

Later phases are expected to move this mixed baseline toward a single coherent
implementation with a narrower unsafe boundary:

- reduce the 208 upstream-owned public exports by porting whole ABI families to
  Rust or by introducing intentionally minimal C shims where longjmp behavior
  still has to stay on the C side
- eliminate the current `libpng16_upstream.a` fallback for public ABI owners
- remove private-layout mirroring once Rust no longer depends on upstream
  private struct access through the shim
- delete or intentionally rewire the dormant wrapper modules instead of keeping
  parallel candidate implementations in the tree
- keep the current non-APNG contract intact unless the Debian patch series is
  intentionally changed; this phase does not apply
  `original/debian/patches/libpng-1.6.39-apng.patch`
