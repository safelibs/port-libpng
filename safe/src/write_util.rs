//! Force-link the upstream write-side utility and chunk configuration entry points.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_write_sig();
    fn png_write_chunk();
    fn png_write_chunk_start();
    fn png_write_chunk_data();
    fn png_write_chunk_end();
    fn png_save_uint_16();
    fn png_save_uint_32();
    fn png_set_compression_buffer_size();
    fn png_set_compression_level();
    fn png_set_compression_mem_level();
    fn png_set_compression_method();
    fn png_set_compression_strategy();
    fn png_set_compression_window_bits();
    fn png_set_text_compression_level();
    fn png_set_text_compression_mem_level();
    fn png_set_text_compression_method();
    fn png_set_text_compression_strategy();
    fn png_set_text_compression_window_bits();
}

#[used]
static FORCE_LINK_WRITE_UTIL: [KeepSymbol; 18] = [
    KeepSymbol(png_write_sig as *const ()),
    KeepSymbol(png_write_chunk as *const ()),
    KeepSymbol(png_write_chunk_start as *const ()),
    KeepSymbol(png_write_chunk_data as *const ()),
    KeepSymbol(png_write_chunk_end as *const ()),
    KeepSymbol(png_save_uint_16 as *const ()),
    KeepSymbol(png_save_uint_32 as *const ()),
    KeepSymbol(png_set_compression_buffer_size as *const ()),
    KeepSymbol(png_set_compression_level as *const ()),
    KeepSymbol(png_set_compression_mem_level as *const ()),
    KeepSymbol(png_set_compression_method as *const ()),
    KeepSymbol(png_set_compression_strategy as *const ()),
    KeepSymbol(png_set_compression_window_bits as *const ()),
    KeepSymbol(png_set_text_compression_level as *const ()),
    KeepSymbol(png_set_text_compression_mem_level as *const ()),
    KeepSymbol(png_set_text_compression_method as *const ()),
    KeepSymbol(png_set_text_compression_strategy as *const ()),
    KeepSymbol(png_set_text_compression_window_bits as *const ()),
];
