# Unsafe Boundary

This phase treats the shipped `libpng16` ABI surface as Rust-owned. The public
entry points live in `safe/src/` and are exported from Rust. Unsafe code remains
only where the C ABI or compiler-private layout rules make a direct Rust
replacement impractical.

## Checked-In C Boundary

The checked-in C sources are intentionally narrow:

- `safe/cshim/longjmp_bridge.c`
  Exposes the final `jmp_buf` field ABI used by `png_set_longjmp_fn()` and
  `png_longjmp()`. This is the only checked-in code that directly touches the
  libpng `jmp_buf` storage fields.

- `safe/cshim/read_phase_bridge.c`
  Defines the mirror structs and layout-copy accessor declarations used to move
  private read-core state between Rust and libpng-private layouts. It does not
  own public parser or writer semantics.

## Rust Unsafe Surface

The remaining unsafe Rust code falls into four buckets:

- Public `extern "C"` exports in `safe/src/`
  These functions receive foreign pointers and must validate nullability, alias
  assumptions, and panic containment before crossing back into C.

- FFI declarations in `safe/src/bridge_ffi.rs`
  These declare the internal support symbols Rust uses to reach the remaining
  non-public ABI glue without re-describing libpng internals in every module.

- Layout mirror copies in `safe/src/chunks.rs`, `safe/src/read.rs`, and related
  read-core helpers
  These pass `png_safe_read_core` / `png_safe_info_core` mirrors across the C
  boundary so Rust can own parser flow without directly spelling the private
  `png_struct` / `png_info` layout.

- zlib bindings in `safe/src/zlib.rs`
  Inflate state remains an FFI boundary because the shipped library still links
  against the platform zlib ABI.

## Invariants

- Rust owns the public export surface and panic containment.
- Checked-in C does not own public read, write, simplified, or transform entry
  point semantics.
- `png_struct` and `png_info` pointers remain opaque to Rust outside the mirror
  copies and `jmp_buf` field accessors.
- APNG remains out of scope. The frozen
  `original/debian/patches/libpng-1.6.39-apng.patch` artifact stays unapplied.
