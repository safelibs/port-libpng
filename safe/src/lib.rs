#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod abi_exports;
mod chunks;
mod colorspace;
mod interlace;
mod read;
mod read_progressive;
mod read_transform;
mod read_util;
mod simplified;
mod types;
mod write;
mod write_transform;
mod write_util;
mod zlib;
