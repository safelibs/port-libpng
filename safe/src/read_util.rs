//! Force-link the upstream core utility objects into the final library.

#[repr(transparent)]
pub(crate) struct KeepSymbol(pub *const ());

unsafe impl Sync for KeepSymbol {}

unsafe extern "C" {
    fn png_access_version_number() -> u32;
    fn png_warning();
    fn png_get_IHDR();
    fn png_free();
}

#[used]
static FORCE_LINK_CORE: [KeepSymbol; 4] = [
    KeepSymbol(png_access_version_number as *const ()),
    KeepSymbol(png_warning as *const ()),
    KeepSymbol(png_get_IHDR as *const ()),
    KeepSymbol(png_free as *const ()),
];
