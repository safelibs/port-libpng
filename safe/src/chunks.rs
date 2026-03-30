//! Force-link the upstream chunk metadata objects into the final library.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_set_keep_unknown_chunks();
    fn png_write_chunk();
}

#[used]
static FORCE_LINK_CHUNKS: [KeepSymbol; 2] = [
    KeepSymbol(png_set_keep_unknown_chunks as *const ()),
    KeepSymbol(png_write_chunk as *const ()),
];
