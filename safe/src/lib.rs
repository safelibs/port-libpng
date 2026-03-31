#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod abi_exports;
mod chunks;
mod colorspace;
mod common;
mod error;
mod get;
mod interlace;
mod io;
mod memory;
// Internal compatibility bindings are generated into OUT_DIR during the build.
mod bridge_ffi;
mod read;
mod read_progressive;
mod read_transform;
mod read_util;
mod set;
mod simplified;
mod state;
mod types;
mod write;
mod write_transform;
mod write_util;
mod zlib;
