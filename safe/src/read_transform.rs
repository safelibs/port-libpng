//! Force-link the upstream read transform and row-update objects into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_read_info();
    fn png_read_update_info();
    fn png_read_png();
    fn png_set_quantize();
    fn png_set_check_for_invalid_index();
}

#[used]
static FORCE_LINK_READ_TRANSFORMS: [KeepSymbol; 5] = [
    KeepSymbol(png_read_info as *const ()),
    KeepSymbol(png_read_update_info as *const ()),
    KeepSymbol(png_read_png as *const ()),
    KeepSymbol(png_set_quantize as *const ()),
    KeepSymbol(png_set_check_for_invalid_index as *const ()),
];
