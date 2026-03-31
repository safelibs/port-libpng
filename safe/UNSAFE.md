# Unsafe Boundary

The exported `libpng16` ABI is implemented in Rust under `safe/src/`. Unsafe
code remains where the C ABI crosses into Rust, where foreign callbacks are
invoked, and where zlib or `jmp_buf` interop cannot be expressed as safe Rust.

## Checked-In C Boundary

The checked-in C files under `safe/cshim/` are retained as documentation of the
final narrowed interop surface, but the phase no longer builds or ships a
hidden libpng runtime from vendor C translation units.

## Unsafe Rust Surface

- Public `extern "C"` exports in `safe/src/`
  These accept foreign pointers, contain panics, and uphold the libpng ABI.

- Opaque handle ownership in `safe/src/memory.rs` and `safe/src/state.rs`
  Rust allocates and tracks the exported `png_struct` / `png_info` handles and
  the internal state attached to them.

- Internal bridge helpers in `safe/src/bridge_ffi.rs`
  Remaining internal bridge names are implemented in Rust instead of being
  generated from upstream C sources at build time.

- Callback, file I/O, and longjmp interop in `safe/src/io.rs` and `safe/src/error.rs`
  These modules cross into application callbacks, stdio, and opaque jmp buffer
  storage.

- zlib interop in `safe/src/zlib.rs`
  The packaged library still links against the platform zlib ABI.

## Invariants

- The build no longer compiles `original/png*.c`, wrapped vendor payloads, or
  generated vendor translation units as part of the shipped runtime.
- Rust owns the exported ABI entry points and the opaque handle lifetime.
- Any remaining C interop is limited to the narrow checked-in shim artifacts and
  does not implement public read, write, simplified, transform, getter, or
  setter semantics.
- APNG remains out of scope. The frozen
  `original/debian/patches/libpng-1.6.39-apng.patch` artifact stays unapplied.
