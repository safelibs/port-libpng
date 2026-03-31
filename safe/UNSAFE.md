# Unsafe Boundaries

This build is still a mixed Rust and upstream-C libpng implementation.

- The final shared library exports the Ubuntu-compatible 246-symbol ABI.
- `safe/build.rs` compiles the upstream C sources into `libpng16_upstream.a`.
- For the APIs now owned by Rust, `safe/build.rs` renames the upstream duplicate
  definitions to `upstream_*` so the public symbol resolves to the Rust export.
- The remaining parser, sequential-read, progressive-read, and write core still
  ship from the upstream C objects.

## Rust-Owned Export Surface

The compiled crate root in [safe/src/lib.rs](/home/yans/code/safelibs/ported/libpng/safe/src/lib.rs)
now wires these Rust implementation modules into the shipped ABI:

- [safe/src/common.rs](/home/yans/code/safelibs/ported/libpng/safe/src/common.rs):
  version helpers, integer save or parse helpers, RFC1123 helpers, and shared
  utility exports.
- [safe/src/error.rs](/home/yans/code/safelibs/ported/libpng/safe/src/error.rs):
  warnings, fatal errors, callback registration, and `png_set_longjmp_fn` or
  `png_longjmp`.
- [safe/src/memory.rs](/home/yans/code/safelibs/ported/libpng/safe/src/memory.rs):
  create or destroy paths, allocator hooks, and info-structure ownership.
- [safe/src/io.rs](/home/yans/code/safelibs/ported/libpng/safe/src/io.rs):
  read or write callback registration and callback metadata accessors.
- [safe/src/get.rs](/home/yans/code/safelibs/ported/libpng/safe/src/get.rs):
  getter family for image metadata, limits, and row layout.
- [safe/src/set.rs](/home/yans/code/safelibs/ported/libpng/safe/src/set.rs):
  setter family for limits, rows, options, and benign-error policy.
- [safe/src/read_transform.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_transform.rs):
  read-transform configuration plus the Rust-owned `png_read_row` wrapper.
- [safe/src/colorspace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/colorspace.rs):
  colorspace, background, gamma-mode, and cHRM XYZ exports.
- [safe/src/interlace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/interlace.rs):
  `png_set_interlace_handling`.
- [safe/src/simplified.rs](/home/yans/code/safelibs/ported/libpng/safe/src/simplified.rs):
  `png_image_*` panic-guarded trampolines to the renamed upstream simplified
  implementation.

At the time of this phase, that compiled Rust set owns 114 exported `png_*`
entry points. The remaining exported ABI continues to come from upstream C
objects linked into the same library.

## Upstream-C Surface Still Shipped

These modules exist only to keep the unported upstream objects linked into the
final library:

- [safe/src/read_util.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_util.rs)
- [safe/src/read_progressive.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_progressive.rs)
- [safe/src/write.rs](/home/yans/code/safelibs/ported/libpng/safe/src/write.rs)
- [safe/src/write_transform.rs](/home/yans/code/safelibs/ported/libpng/safe/src/write_transform.rs)
- [safe/src/write_util.rs](/home/yans/code/safelibs/ported/libpng/safe/src/write_util.rs)
- [safe/src/zlib.rs](/home/yans/code/safelibs/ported/libpng/safe/src/zlib.rs)

They do not implement new logic. They retain upstream objects by storing C
function addresses in `#[used]` statics.

## Unavoidable Unsafe In The Compiled Rust Surface

The remaining compiled `unsafe` is restricted to the boundaries Rust cannot
model without changing the libpng ABI contract:

- C ABI entry points:
  exported `extern "C"` functions accept raw `png_struct*`, `png_info*`,
  caller buffers, and callback pointers from foreign code.
- Upstream trampoline calls:
  [safe/src/memory.rs](/home/yans/code/safelibs/ported/libpng/safe/src/memory.rs),
  [safe/src/io.rs](/home/yans/code/safelibs/ported/libpng/safe/src/io.rs),
  [safe/src/get.rs](/home/yans/code/safelibs/ported/libpng/safe/src/get.rs),
  [safe/src/set.rs](/home/yans/code/safelibs/ported/libpng/safe/src/set.rs),
  [safe/src/error.rs](/home/yans/code/safelibs/ported/libpng/safe/src/error.rs),
  [safe/src/common.rs](/home/yans/code/safelibs/ported/libpng/safe/src/common.rs),
  and [safe/src/simplified.rs](/home/yans/code/safelibs/ported/libpng/safe/src/simplified.rs)
  call renamed `upstream_*` functions through FFI. That requires raw pointers
  and foreign function signatures on both the entry-point and callee sides.
- Foreign allocator ownership:
  [safe/src/memory.rs](/home/yans/code/safelibs/ported/libpng/safe/src/memory.rs)
  forwards allocator registration, allocation, and free calls across the C ABI;
  allocator identity and ownership are controlled by the foreign caller.
- Callback-pointer interop:
  [safe/src/io.rs](/home/yans/code/safelibs/ported/libpng/safe/src/io.rs) and
  [safe/src/error.rs](/home/yans/code/safelibs/ported/libpng/safe/src/error.rs)
  forward callback pointers and callback state that Rust cannot type-check for
  aliasing, lifetime, or unwind behavior.
- Raw buffer and struct-pointer access:
  [safe/src/common.rs](/home/yans/code/safelibs/ported/libpng/safe/src/common.rs),
  [safe/src/colorspace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/colorspace.rs),
  [safe/src/interlace.rs](/home/yans/code/safelibs/ported/libpng/safe/src/interlace.rs),
  [safe/src/chunks.rs](/home/yans/code/safelibs/ported/libpng/safe/src/chunks.rs),
  and [safe/src/read_transform.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read_transform.rs)
  dereference caller-provided pointers or reinterpret row buffers in order to
  preserve the libpng C ABI.
- Read-phase C bridge:
  [safe/cshim/read_phase_bridge.c](/home/yans/code/safelibs/ported/libpng/safe/cshim/read_phase_bridge.c)
  is the only active compiled C shim. It wraps upstream read-path calls with
  `setjmp` recovery and also mirrors selected private `png_struct` and
  `png_info` fields into compact snapshot structs so the Rust read-transform
  layer can inspect and restore layout-sensitive upstream state without
  reimplementing the full parser in Rust.

## Explicit Non-Goals And Exclusions

- There is no remaining custom `unsafe impl` for linker-retention statics.
  `KeepSymbol` is now `AtomicPtr<()>`, so the old unjustified `unsafe impl Sync`
  is gone.
- The old manual simplified-reader experiment was removed from the compiled
  surface. [safe/src/simplified.rs](/home/yans/code/safelibs/ported/libpng/safe/src/simplified.rs)
  now contains only the active exported trampolines.
- [safe/src/read.rs](/home/yans/code/safelibs/ported/libpng/safe/src/read.rs)
  remains in the tree as an inactive shim experiment, but it is not wired into
  [safe/src/lib.rs](/home/yans/code/safelibs/ported/libpng/safe/src/lib.rs) and
  is not part of the shipped ABI in this phase.
- [safe/src/state.rs](/home/yans/code/safelibs/ported/libpng/safe/src/state.rs)
  also remains only as a dormant reference file and is not compiled into the
  shipped build.
- [safe/cshim/longjmp_bridge.c](/home/yans/code/safelibs/ported/libpng/safe/cshim/longjmp_bridge.c)
  also remains in the tree only as a dormant experiment and is no longer
  compiled into the shipped build.

Any new `unsafe` in compiled Rust should fit one of the categories above. If it
does not, it should be removed or documented with a narrower justification.
