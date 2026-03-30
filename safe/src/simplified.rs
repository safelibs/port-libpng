//! Force-link the upstream simplified read entry points into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_image_begin_read_from_file();
    fn png_image_begin_read_from_stdio();
    fn png_image_begin_read_from_memory();
    fn png_image_finish_read();
    fn png_image_free();
}

#[used]
static FORCE_LINK_SIMPLIFIED_READ: [KeepSymbol; 5] = [
    KeepSymbol(png_image_begin_read_from_file as *const ()),
    KeepSymbol(png_image_begin_read_from_stdio as *const ()),
    KeepSymbol(png_image_begin_read_from_memory as *const ()),
    KeepSymbol(png_image_finish_read as *const ()),
    KeepSymbol(png_image_free as *const ()),
];
