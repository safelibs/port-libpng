#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_op_in_unsafe_fn)]

#[macro_export]
macro_rules! abi_guard {
    ($png_ptr:expr, $body:expr) => {{
        let _ = $png_ptr;
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(value) => value,
            Err(_) => std::process::abort(),
        }
    }};
}

#[macro_export]
macro_rules! abi_guard_no_png {
    ($body:expr) => {{
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(value) => value,
            Err(_) => std::process::abort(),
        }
    }};
}

pub mod abi_exports;
mod chunks;
mod colorspace;
mod interlace;
mod read;
mod read_progressive;
mod read_transform;
mod read_util;
mod types;
mod write;
mod write_transform;
mod write_util;
mod zlib;
