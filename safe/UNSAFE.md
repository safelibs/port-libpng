# Unsafe Boundaries

The remaining `unsafe` in the Phase 8 tree is limited to ABI and runtime edges
that Rust cannot express safely without changing the libpng contract.

## FFI and ABI edges

- `safe/src/lib.rs` exports a C ABI and must accept raw pointers, C enums, and
  callback function pointers from untrusted callers.
- `safe/src/chunks.rs`, `safe/src/read.rs`, `safe/src/read_transform.rs`,
  `safe/src/colorspace.rs`, `safe/src/interlace.rs`, and
  `safe/src/simplified.rs` call into upstream libpng C objects or C shims and
  therefore cross raw-pointer boundaries.
- `safe/src/state.rs` reinterprets `png_struct` and `png_info` storage as
  Rust-side state mirrors with `#[repr(C)]` layouts.

## Callback and longjmp interop

- `safe/cshim/longjmp_bridge.c` is required because `jmp_buf`, `setjmp`, and
  `longjmp` are not portable to model directly in Rust.
- `safe/cshim/read_phase_bridge.c` wraps upstream read-phase entry points that
  can `longjmp` and snapshots fields Rust needs without exposing libpng private
  layouts directly to safe Rust.
- `safe/src/simplified.rs` and the read-side modules still store raw callback
  pointers and opaque context pointers supplied by C callers.

## Allocation and buffer ownership

- `safe/src/simplified.rs` writes directly into caller-provided output buffers.
  The API contract is inherited from libpng, so Rust cannot prove the buffer
  size or stride is correct without runtime checks plus raw-pointer writes.
- Any path that invokes user allocator hooks or frees memory through caller
  callbacks must remain `unsafe` because Rust cannot verify foreign ownership or
  allocator identity at compile time.

## Hardening stance

- Internal arithmetic and state transitions are kept in safe Rust where
  possible.
- Exported entry points now wrap the remaining Rust logic in panic guards so
  panics abort instead of unwinding across the C ABI.
- New CVE coverage under `safe/tests/cve-regressions/` documents which bug
  classes are exercised by tests versus enforced by checked arithmetic or state
  invariants.
