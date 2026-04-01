# Unsafe Boundary

The exported `libpng16` ABI is implemented in Rust under `safe/src/`. Unsafe
code remains where the C ABI crosses into Rust, where foreign callbacks are
invoked, and where zlib or `jmp_buf` interop cannot be expressed as safe Rust.

## Checked-In C Boundary

The checked-in C files under `safe/cshim/` are limited to the final narrow
interop boundary:

- `safe/cshim/longjmp_bridge.c`
  Documents the `jmp_buf` field ABI and the longjmp callback handoff.

- `safe/cshim/read_phase_bridge.c`
  Documents the remaining private-layout mirror types used for read-side field
  synchronization.

These bridge artifacts do not ship an alternate libpng runtime and do not
reintroduce `original/png*.c` as packaged implementation code.

## Unsafe Rust Surface

- Public `extern "C"` exports in `safe/src/`
  These accept foreign pointers, contain panics, and uphold the libpng ABI.

- Opaque handle ownership in `safe/src/memory.rs` and `safe/src/state.rs`
  Rust allocates and tracks the exported `png_struct` / `png_info` handles and
  the internal state attached to them.

- Internal bridge helpers in `safe/src/bridge_ffi.rs`
  Remaining internal bridge names are implemented in Rust instead of being
  generated from upstream C sources at build time.

- Callback, file I/O, and longjmp interop in `safe/src/io.rs`, `safe/src/error.rs`,
  and `safe/src/memory.rs`
  These modules cross into application callbacks, stdio, opaque jmp buffer
  storage, and the create-time `_setjmp` / `longjmp` trap needed to preserve
  libpng's user-allocator failure semantics.

- zlib interop in `safe/src/zlib.rs`
  The packaged library still links against the platform zlib ABI.

## Invariants

- The build no longer compiles `original/png*.c`, wrapped vendor payloads, or
  generated vendor translation units as part of the shipped runtime.
- Rust owns the exported ABI entry points and the opaque handle lifetime.
- Any remaining C interop is limited to the narrow checked-in shim artifacts
  needed for `jmp_buf`, callback, or private-layout bridge glue and does not
  implement public read, write, simplified, transform, getter, or setter
  semantics.
- APNG remains out of scope. The frozen
  `original/debian/patches/libpng-1.6.39-apng.patch` artifact stays unapplied.
