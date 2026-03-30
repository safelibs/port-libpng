//! Force-link the upstream write-side construction objects used by existing core smoke.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_create_write_struct();
    fn png_set_write_fn();
}

#[used]
static FORCE_LINK_WRITE_CORE: [KeepSymbol; 2] = [
    KeepSymbol(png_create_write_struct as *const ()),
    KeepSymbol(png_set_write_fn as *const ()),
];
