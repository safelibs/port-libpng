//! Force-link the upstream interlace and transform objects into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_set_interlace_handling();
    fn png_set_expand();
}

#[used]
static FORCE_LINK_INTERLACE: [KeepSymbol; 2] = [
    KeepSymbol(png_set_interlace_handling as *const ()),
    KeepSymbol(png_set_expand as *const ()),
];
