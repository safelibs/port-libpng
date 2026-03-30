//! Force-link the upstream colorspace setters/getters exercised by read-side transforms.

use crate::read_util::KeepSymbol;

unsafe extern "C" {
    fn png_get_cHRM_XYZ_fixed();
    fn png_set_cHRM_XYZ_fixed();
    fn png_set_rgb_to_gray_fixed();
    fn png_set_background_fixed();
    fn png_set_alpha_mode_fixed();
}

#[used]
static FORCE_LINK_COLORSPACE: [KeepSymbol; 5] = [
    KeepSymbol(png_get_cHRM_XYZ_fixed as *const ()),
    KeepSymbol(png_set_cHRM_XYZ_fixed as *const ()),
    KeepSymbol(png_set_rgb_to_gray_fixed as *const ()),
    KeepSymbol(png_set_background_fixed as *const ()),
    KeepSymbol(png_set_alpha_mode_fixed as *const ()),
];
