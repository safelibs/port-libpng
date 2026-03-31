# Unsafe Boundaries

This phase still ships a mixed Rust and upstream-C libpng. The active boundary
is now broader than the earlier read-transform-only baseline: Rust owns the
public core lifetime, callback registration, error, memory, IO, and policy
entry points, while upstream C still owns the underlying `png_struct` and
`png_info` storage layout plus the remaining read/write execution paths.

## Current Hybrid Baseline

- `safe/build.rs` still compiles the upstream core libpng C sources from
  `original/` into `libpng16_upstream.a`.
- The frozen public ABI still exports 246 `png_*` symbols
  (`safe/abi/exports.txt`).
- `UPSTREAM_RENAMES` in `safe/build.rs` now renames 95 public symbols to
  `upstream_*` before the upstream C objects are linked.
- 94 of those 95 renamed symbols are re-owned by Rust.
- 1 renamed symbol, `png_read_row`, remains re-owned by
  `safe/cshim/read_phase_bridge.c`.
- The remaining 151 public `png_*` exports are still upstream-C owned.

## Active Rust-Owned Public ABI

The compiled crate root now includes the core modules instead of leaving them
dormant:

- existing phase-1 owners:
  `safe/src/common.rs`, `safe/src/read_transform.rs`,
  `safe/src/interlace.rs`, and `safe/src/colorspace.rs`
- newly active phase-2 owners:
  `safe/src/error.rs`, `safe/src/memory.rs`, `safe/src/io.rs`,
  `safe/src/get.rs`, `safe/src/set.rs`, and `safe/src/state.rs`
- compiled support modules:
  `safe/src/chunks.rs`, `safe/src/read_progressive.rs`,
  `safe/src/read_util.rs`, `safe/src/write.rs`,
  `safe/src/write_transform.rs`, `safe/src/write_util.rs`,
  `safe/src/zlib.rs`, `safe/src/types.rs`, and `safe/src/abi_exports.rs`

The newly Rust-owned public symbol families are:

- error and longjmp registration:
  `png_warning`, `png_error`, `png_benign_error`, `png_chunk_warning`,
  `png_chunk_error`, `png_chunk_benign_error`, `png_set_error_fn`,
  `png_get_error_ptr`, `png_set_longjmp_fn`, and `png_longjmp`
- allocator and lifetime surface:
  `png_calloc`, `png_malloc`, `png_malloc_default`, `png_malloc_warn`,
  `png_free`, `png_free_default`, `png_set_mem_fn`, `png_get_mem_ptr`,
  `png_create_read_struct[_2]`, `png_create_write_struct[_2]`,
  `png_create_info_struct`, `png_destroy_read_struct`,
  `png_destroy_write_struct`, `png_destroy_info_struct`,
  `png_info_init_3`, `png_data_freer`, and `png_free_data`
- IO and callback registration:
  `png_init_io`, `png_get_io_ptr`, `png_set_read_fn`, `png_set_write_fn`,
  `png_set_read_status_fn`, `png_set_write_status_fn`,
  `png_set_progressive_read_fn`, `png_get_progressive_ptr`,
  `png_set_read_user_chunk_fn`, `png_get_user_chunk_ptr`,
  `png_set_read_user_transform_fn`, `png_set_write_user_transform_fn`,
  `png_set_user_transform_info`, `png_get_user_transform_ptr`,
  `png_get_io_state`, and `png_get_io_chunk_type`
- core getters and setters:
  `png_set_sig_bytes`, `png_get_valid`, `png_get_rowbytes`, `png_get_rows`,
  `png_set_rows`, `png_get_channels`, `png_get_image_width`,
  `png_get_image_height`, `png_get_bit_depth`, `png_get_color_type`,
  `png_get_filter_type`, `png_get_interlace_type`,
  `png_get_compression_type`, `png_set_user_limits`,
  `png_get_user_width_max`, `png_get_user_height_max`,
  `png_set_chunk_cache_max`, `png_get_chunk_cache_max`,
  `png_set_chunk_malloc_max`, `png_get_chunk_malloc_max`,
  `png_set_benign_errors`, `png_set_check_for_invalid_index`,
  `png_get_palette_max`, and `png_set_option`

## Mixed-Runtime Object Ownership

During phases 2 through 5, Rust does not replace the concrete `png_struct` or
`png_info` representation. The active object model is:

- live `png_struct*` and `png_info*` objects are still allocated as
  upstream-compatible storage by the renamed upstream constructors and
  destructors
- `safe/src/state.rs` attaches Rust sidecar state to those live pointers and
  mirrors callback registrations, user payloads, limits, option bits, longjmp
  metadata, and `png_info` ownership flags
- setters update both worlds:
  the actual upstream-compatible struct fields via the renamed upstream helper
  functions, and the Rust sidecar for later Rust-owned phases
- getters prefer the Rust sidecar only where the state is purely
  registration-policy data and fall back to upstream field access when the
  upstream runtime may legitimately mutate the underlying fields

This mixed model is intentional. Later phases can consume the Rust-owned sidecar
state without breaking the still-upstream read/write execution code that
expects the original private layouts.

## Active Longjmp Boundary

`safe/cshim/longjmp_bridge.c` is now the authoritative `jmp_buf` storage and
interop boundary for the public longjmp APIs:

- Rust-owned `png_set_longjmp_fn` uses the shim to discover the local
  `jmp_buf`, populate the active `png_struct` longjmp fields, and preserve the
  upstream size-mismatch behavior
- Rust-owned `png_longjmp` delegates the actual callback invocation through the
  shim and aborts if no valid longjmp target is registered
- the public `png_jmpbuf` macro in the shipped headers now lands on the
  Rust-owned `png_set_longjmp_fn`, but still receives a real `jmp_buf *`
  compatible with application `setjmp`

`safe/cshim/read_phase_bridge.c` remains compiled, but only for the
still-upstream read-path containment and private-layout mirror helpers used by
the existing read-transform work. It is no longer the active longjmp boundary
for the public core ABI.

## Remaining Upstream-Owned Public ABI

The following major surfaces are still upstream-owned in this phase:

- sequential read execution, progressive read execution, and the remaining
  read-path parser/chunk helpers not already ported
- write execution, chunk emission, compression-control, and simplified write
  entry points
- simplified `png_image_*` helpers
- metadata setters/getters and chunk helpers outside the phase-1 and phase-2
  families listed above

## Active Unsafe

The compiled unsafe boundary is now concentrated in four places:

- Rust `extern "C"` exports that receive foreign pointers, buffers, and
  callback function pointers
- raw pointer access inside `safe/src/state.rs` mirror management and the
  Rust-owned ABI wrappers
- FFI from Rust into the active C shims, especially the longjmp field helpers
  and the read-phase bridge
- upstream C code that still owns the remaining execution paths and private
  layout manipulation

The library still aborts on Rust panic at the ABI boundary. Public callbacks
and longjmp behavior follow libpng semantics; Rust panics are not translated
into recoverable libpng errors.

## Non-APNG Contract

This phase preserves the current non-APNG contract. The Debian APNG patch
artifact at `original/debian/patches/libpng-1.6.39-apng.patch` remains
non-applied, and this phase does not introduce APNG ordinals or install-surface
drift.
