//! Force-link the upstream write-side core entry points used in Phase 5.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_create_write_struct();
    fn png_set_write_fn();
    fn png_write_info_before_PLTE();
    fn png_write_info();
    fn png_write_row();
    fn png_write_rows();
    fn png_write_image();
    fn png_write_end();
    fn png_write_png();
    fn png_set_flush();
    fn png_write_flush();
}

#[used]
static FORCE_LINK_WRITE_CORE: [KeepSymbol; 11] = [
    KeepSymbol(png_create_write_struct as *const ()),
    KeepSymbol(png_set_write_fn as *const ()),
    KeepSymbol(png_write_info_before_PLTE as *const ()),
    KeepSymbol(png_write_info as *const ()),
    KeepSymbol(png_write_row as *const ()),
    KeepSymbol(png_write_rows as *const ()),
    KeepSymbol(png_write_image as *const ()),
    KeepSymbol(png_write_end as *const ()),
    KeepSymbol(png_write_png as *const ()),
    KeepSymbol(png_set_flush as *const ()),
    KeepSymbol(png_write_flush as *const ()),
];
