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
    KeepSymbol(png_set_filter as *const ()),
    KeepSymbol(png_set_filter_heuristics as *const ()),
    KeepSymbol(png_set_filter_heuristics_fixed as *const ()),
    KeepSymbol(png_set_add_alpha as *const ()),
    KeepSymbol(png_set_filler as *const ()),
    KeepSymbol(png_set_packing as *const ()),
    KeepSymbol(png_set_packswap as *const ()),
    KeepSymbol(png_set_shift as *const ()),
    KeepSymbol(png_set_swap as *const ()),
    KeepSymbol(png_set_swap_alpha as *const ()),
    KeepSymbol(png_set_invert_alpha as *const ()),
    KeepSymbol(png_set_invert_mono as *const ()),
    KeepSymbol(png_set_bgr as *const ()),
    KeepSymbol(png_set_write_user_transform_fn as *const ()),
    KeepSymbol(png_set_user_transform_info as *const ()),
];
