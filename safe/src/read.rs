//! Force-link the upstream sequential read objects into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_create_read_struct();
    fn png_set_read_fn();
}

#[used]
static FORCE_LINK_READ: [KeepSymbol; 2] = [
    KeepSymbol(png_create_read_struct as *const ()),
    KeepSymbol(png_set_read_fn as *const ()),
];
