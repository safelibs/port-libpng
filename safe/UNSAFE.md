# Unsafe Boundaries

This build still ships a mixed Rust and upstream-C libpng.

- `safe/build.rs` compiles the upstream C libpng sources into `libpng16_upstream.a`.
- The Rust crate only owns a reduced subset of exported `png_*` symbols.
- The rest of the public ABI is now left in upstream C specifically to avoid
  routing upstream `png_error` or `png_longjmp` behavior through Rust frames.

## Rust-Owned Export Surface

The compiled crate root in [safe/src/lib.rs](/home/yans/code/safelibs/ported/libpng/safe/src/lib.rs)
now wires only these Rust implementation modules into the shipped ABI:

- [safe/src/common.rs](/home/yans/code/safelibs/ported/libpng/safe/src/common.rs):
  version strings, signature comparison, integer save helpers, grayscale
  palette generation, and time-conversion helpers. `png_get_uint_31` and
  `png_convert_to_rfc1123` are no longer Rust-owned.
- [safe/src/colorspace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/colorspace.rs):
  `png_set_rgb_to_gray*`, `png_set_background*`, `png_set_alpha_mode*`,
  `png_set_cHRM_XYZ*`, and `png_get_cHRM_XYZ*`.
- [safe/src/interlace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/interlace.rs):
  `png_set_interlace_handling`.
- [safe/src/read_transform.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_transform.rs):
  the read-transform setter family, including `png_set_quantize`.

These Rust exports no longer call renamed upstream functions that can take
fatal `png_error` or `png_longjmp` paths directly through Rust frames.

## Upstream-C Surface Still Shipped

The shipped ABI is still mostly upstream C. In particular, the public symbols
for error handling, longjmp registration, allocator hooks, struct creation or
destruction, IO callback registration, metadata getters, metadata setters,
simplified `png_image_*`, parser core, progressive read core, and write core
remain upstream-owned.

That includes the APIs previously wrapped in dormant Rust forwarding modules:

- [safe/src/error.rs](/home/yans/code/safelibs/ported/libpng/safe/src/error.rs)
- [safe/src/memory.rs](/home/yans/code/safelibs/ported/libpng/safe/src/memory.rs)
- [safe/src/io.rs](/home/yans/code/safelibs/ported/libpng/safe/src/io.rs)
- [safe/src/get.rs](/home/yans/code/safelibs/ported/libpng/safe/src/get.rs)
- [safe/src/set.rs](/home/yans/code/safelibs/ported/libpng/safe/src/set.rs)
- [safe/src/simplified.rs](/home/yans/code/safelibs/ported/libpng/safe/src/simplified.rs)

Those files remain in the tree as reference material, but they are not wired
into [safe/src/lib.rs](/home/yans/code/safelibs/ported/libpng/safe/src/lib.rs)
and are not part of the active Rust implementation surface in this phase.

## Active C Shim Boundary

[safe/cshim/read_phase_bridge.c](/home/yans/code/safelibs/ported/libpng/safe/cshim/read_phase_bridge.c)
is the only active compiled C shim. It remains necessary for three reasons:

- Private-layout mirroring:
  it snapshots and restores selected private `png_struct` and `png_info`
  fields that Rust cannot access portably without depending on upstream
  private layout details.
- Fatal-control-flow containment:
  it wraps selected upstream read-path and helper calls with `setjmp` so
  Rust-owned transform and colorspace exports can trigger libpng fatal-error
  behavior without letting `longjmp` cross Rust frames.
- C-owned `png_read_row` hardening:
  the public `png_read_row` symbol is implemented in this C shim, not in
  Rust. It delegates to `upstream_png_read_row`, then masks trailing packed-row
  padding bits for the hardening regression coverage while keeping any fatal
  libpng control flow entirely on the C side.

The active shim currently contains these longjmp-sensitive helper paths:

- `png_safe_call_read_info`
- `png_safe_call_read_update_info`
- `png_safe_call_read_image`
- `png_safe_call_read_end`
- `png_safe_call_benign_error`
- `png_safe_call_app_error`
- `png_safe_call_error`
- `png_safe_call_set_quantize`
- `png_read_row`

## Remaining Compiled Unsafe In Rust

The remaining compiled Rust `unsafe` is limited to boundary cases that cannot
be expressed safely without changing the libpng ABI:

- `extern "C"` entry points receiving foreign `png_struct*`, `png_info*`,
  caller buffers, and callback pointers.
- Raw pointer dereferences for caller-owned structs and buffers in
  [safe/src/common.rs](/home/yans/code/safelibs/ported/libpng/safe/src/common.rs),
  [safe/src/colorspace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/colorspace.rs),
  [safe/src/interlace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/interlace.rs),
  [safe/src/read_transform.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_transform.rs),
  and [safe/src/chunks.rs](/home/yans/code/safelibs/ported/libpng/safe/src/chunks.rs).
- FFI calls from Rust into the C shim in
  [safe/src/chunks.rs](/home/yans/code/safelibs/ported/libpng/safe/src/chunks.rs)
  for layout snapshots and longjmp-contained helper calls.

Rust exports now use `abi_guard!` or `abi_guard_no_png!` only for panic
containment. On panic they abort the process; they do not translate Rust panic
into `png_error` or `png_longjmp`.

## Dormant Files

These files remain in the repository but are not part of the active compiled
unsafe surface for this phase:

- [safe/src/read.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read.rs)
- [safe/src/state.rs](/home/yans/code/safelibs/ported/libpng/safe/src/state.rs)
- [safe/cshim/longjmp_bridge.c](/home/yans/code/safelibs/ported/libpng/safe/cshim/longjmp_bridge.c)

If new compiled `unsafe` is introduced, it should either stay inside the
documented C ABI or C shim boundaries above or this file must be updated to
justify a narrower active boundary.
