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
    KeepSymbol::new(png_write_sig as *mut ()),
    KeepSymbol::new(png_write_chunk as *mut ()),
    KeepSymbol::new(png_write_chunk_start as *mut ()),
    KeepSymbol::new(png_write_chunk_data as *mut ()),
    KeepSymbol::new(png_write_chunk_end as *mut ()),
    KeepSymbol::new(png_save_uint_16 as *mut ()),
    KeepSymbol::new(png_save_uint_32 as *mut ()),
    KeepSymbol::new(png_set_compression_buffer_size as *mut ()),
    KeepSymbol::new(png_set_compression_level as *mut ()),
    KeepSymbol::new(png_set_compression_mem_level as *mut ()),
    KeepSymbol::new(png_set_compression_method as *mut ()),
    KeepSymbol::new(png_set_compression_strategy as *mut ()),
    KeepSymbol::new(png_set_compression_window_bits as *mut ()),
    KeepSymbol::new(png_set_text_compression_level as *mut ()),
    KeepSymbol::new(png_set_text_compression_mem_level as *mut ()),
    KeepSymbol::new(png_set_text_compression_method as *mut ()),
    KeepSymbol::new(png_set_text_compression_strategy as *mut ()),
    KeepSymbol::new(png_set_text_compression_window_bits as *mut ()),
];
