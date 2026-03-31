use crate::types::*;
use crate::zlib;

unsafe extern "C" {
    fn runtime_png_get_compression_buffer_size(png_ptr: png_const_structrp) -> usize;
    fn runtime_png_write_sig(png_ptr: png_structrp);
    fn runtime_png_write_chunk(
        png_ptr: png_structrp,
        chunk_name: png_const_bytep,
        data: png_const_bytep,
        length: usize,
    );
    fn runtime_png_write_chunk_start(
        png_ptr: png_structrp,
        chunk_name: png_const_bytep,
        length: png_uint_32,
    );
    fn runtime_png_write_chunk_data(png_ptr: png_structrp, data: png_const_bytep, length: usize);
    fn runtime_png_write_chunk_end(png_ptr: png_structrp);
    fn runtime_png_set_compression_buffer_size(png_ptr: png_structrp, size: usize);
    fn runtime_png_set_compression_level(png_ptr: png_structrp, level: core::ffi::c_int);
    fn runtime_png_set_compression_mem_level(png_ptr: png_structrp, mem_level: core::ffi::c_int);
    fn runtime_png_set_compression_method(png_ptr: png_structrp, method: core::ffi::c_int);
    fn runtime_png_set_compression_strategy(png_ptr: png_structrp, strategy: core::ffi::c_int);
    fn runtime_png_set_compression_window_bits(
        png_ptr: png_structrp,
        window_bits: core::ffi::c_int,
    );
    fn runtime_png_set_text_compression_level(png_ptr: png_structrp, level: core::ffi::c_int);
    fn runtime_png_set_text_compression_mem_level(
        png_ptr: png_structrp,
        mem_level: core::ffi::c_int,
    );
    fn runtime_png_set_text_compression_method(png_ptr: png_structrp, method: core::ffi::c_int);
    fn runtime_png_set_text_compression_strategy(
        png_ptr: png_structrp,
        strategy: core::ffi::c_int,
    );
    fn runtime_png_set_text_compression_window_bits(
        png_ptr: png_structrp,
        window_bits: core::ffi::c_int,
    );
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_get_compression_buffer_size(png_ptr: png_const_structrp) -> usize {
    crate::abi_guard!(png_ptr.cast_mut(), {
        zlib::write_zlib_settings(png_ptr)
            .map(|settings| settings.buffer_size)
            .unwrap_or_else(|| unsafe { runtime_png_get_compression_buffer_size(png_ptr) })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_sig(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_write_sig(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_chunk(
    png_ptr: png_structrp,
    chunk_name: png_const_bytep,
    data: png_const_bytep,
    length: usize,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_write_chunk(png_ptr, chunk_name, data, length);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_chunk_start(
    png_ptr: png_structrp,
    chunk_name: png_const_bytep,
    length: png_uint_32,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_write_chunk_start(png_ptr, chunk_name, length);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_chunk_data(
    png_ptr: png_structrp,
    data: png_const_bytep,
    length: usize,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_write_chunk_data(png_ptr, data, length);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_write_chunk_end(png_ptr: png_structrp) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_write_chunk_end(png_ptr);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_buffer_size(png_ptr: png_structrp, size: usize) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_buffer_size(png_ptr, size);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.buffer_size = size;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_level(png_ptr: png_structrp, level: core::ffi::c_int) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_level(png_ptr, level);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.level = level;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_mem_level(
    png_ptr: png_structrp,
    mem_level: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_mem_level(png_ptr, mem_level);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.mem_level = mem_level;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_method(
    png_ptr: png_structrp,
    method: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_method(png_ptr, method);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.method = method;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_strategy(
    png_ptr: png_structrp,
    strategy: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_strategy(png_ptr, strategy);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.strategy = strategy;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_compression_window_bits(
    png_ptr: png_structrp,
    window_bits: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_compression_window_bits(png_ptr, window_bits);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.window_bits = window_bits;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_text_compression_level(
    png_ptr: png_structrp,
    level: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_text_compression_level(png_ptr, level);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.text_level = level;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_text_compression_mem_level(
    png_ptr: png_structrp,
    mem_level: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_text_compression_mem_level(png_ptr, mem_level);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.text_mem_level = mem_level;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_text_compression_method(
    png_ptr: png_structrp,
    method: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_text_compression_method(png_ptr, method);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.text_method = method;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_text_compression_strategy(
    png_ptr: png_structrp,
    strategy: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_text_compression_strategy(png_ptr, strategy);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.text_strategy = strategy;
        });
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn png_set_text_compression_window_bits(
    png_ptr: png_structrp,
    window_bits: core::ffi::c_int,
) {
    crate::abi_guard!(png_ptr, unsafe {
        runtime_png_set_text_compression_window_bits(png_ptr, window_bits);
        zlib::update_write_zlib_settings(png_ptr, |settings| {
            settings.text_window_bits = window_bits;
        });
    });
}
