# Unsafe Boundary

The exported `libpng16` ABI is owned from Rust in `safe/src/`. Unsafe code is
kept at the points where the C ABI, `jmp_buf`, or private libpng layout copies
cannot be represented directly in stable Rust.

## Checked-In C Boundary

The checked-in C sources are intentionally limited to the final ABI shims:

- `safe/cshim/longjmp_bridge.c`
  Accesses the `jmp_buf` storage fields used by `png_set_longjmp_fn()` and
  `png_longjmp()`.

- `safe/cshim/read_phase_bridge.c`
  Defines the mirror structs and layout-copy declarations used for private
  read-core snapshots.

These files are the only checked-in C sources that participate in the final
unsafe boundary.

## Unsafe Rust Surface

- Public `extern "C"` exports in `safe/src/`
  These accept foreign pointers, contain panics, and uphold the libpng C ABI.

- Build-generated internal bindings
  Rust still declares a small internal compatibility surface in generated
  bindings emitted under `OUT_DIR`, so checked-in modules do not duplicate
  private ABI declarations.

- Layout mirror copies in the read core
  `safe/src/chunks.rs`, `safe/src/read.rs`, and related helpers exchange
  `png_safe_read_core` / `png_safe_info_core` mirrors with the private layout
  bridge.

- zlib interop in `safe/src/zlib.rs`
  The packaged library continues to link against the platform zlib ABI.

## Invariants

- Rust owns the exported ABI entry points and their panic containment.
- Checked-in C does not define public read, write, simplified, or transform
  entry points.
- `png_struct` and `png_info` remain opaque outside the mirror copies and the
  final `jmp_buf` access shim.
- APNG remains out of scope. The frozen
  `original/debian/patches/libpng-1.6.39-apng.patch` artifact stays unapplied.
