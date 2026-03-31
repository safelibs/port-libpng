//! Force-link the upstream write-side core entry points used in Phase 5.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
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
static FORCE_LINK_WRITE_CORE: [KeepSymbol; 9] = [
    KeepSymbol::new(png_write_info_before_PLTE as *mut ()),
    KeepSymbol::new(png_write_info as *mut ()),
    KeepSymbol::new(png_write_row as *mut ()),
    KeepSymbol::new(png_write_rows as *mut ()),
    KeepSymbol::new(png_write_image as *mut ()),
    KeepSymbol::new(png_write_end as *mut ()),
    KeepSymbol::new(png_write_png as *mut ()),
    KeepSymbol::new(png_set_flush as *mut ()),
    KeepSymbol::new(png_write_flush as *mut ()),
];
