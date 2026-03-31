//! Force-link the upstream write-side transform and filter controls.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_set_filter();
    fn png_set_filter_heuristics();
    fn png_set_filter_heuristics_fixed();
    fn png_set_add_alpha();
    fn png_set_filler();
    fn png_set_packing();
    fn png_set_packswap();
    fn png_set_shift();
    fn png_set_swap();
    fn png_set_swap_alpha();
    fn png_set_invert_alpha();
    fn png_set_invert_mono();
    fn png_set_bgr();
    fn png_set_write_user_transform_fn();
    fn png_set_user_transform_info();
}

#[used]
static FORCE_LINK_WRITE_TRANSFORMS: [KeepSymbol; 15] = [
    KeepSymbol::new(png_set_filter as *mut ()),
    KeepSymbol::new(png_set_filter_heuristics as *mut ()),
    KeepSymbol::new(png_set_filter_heuristics_fixed as *mut ()),
    KeepSymbol::new(png_set_add_alpha as *mut ()),
    KeepSymbol::new(png_set_filler as *mut ()),
    KeepSymbol::new(png_set_packing as *mut ()),
    KeepSymbol::new(png_set_packswap as *mut ()),
    KeepSymbol::new(png_set_shift as *mut ()),
    KeepSymbol::new(png_set_swap as *mut ()),
    KeepSymbol::new(png_set_swap_alpha as *mut ()),
    KeepSymbol::new(png_set_invert_alpha as *mut ()),
    KeepSymbol::new(png_set_invert_mono as *mut ()),
    KeepSymbol::new(png_set_bgr as *mut ()),
    KeepSymbol::new(png_set_write_user_transform_fn as *mut ()),
    KeepSymbol::new(png_set_user_transform_info as *mut ()),
];
