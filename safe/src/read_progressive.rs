//! Force-link the upstream progressive read object into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_process_data();
}

#[used]
static FORCE_LINK_PROGRESSIVE: KeepSymbol = KeepSymbol::new(png_process_data as *mut ());
