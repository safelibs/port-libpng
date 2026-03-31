//! Force-link the upstream core utility objects into the final library.

pub(crate) type KeepSymbol = core::sync::atomic::AtomicPtr<()>;

unsafe extern "C" {
    fn png_access_version_number() -> u32;
    fn png_warning();
    fn png_get_IHDR();
    fn png_free();
}

#[used]
static FORCE_LINK_CORE: [KeepSymbol; 4] = [
    KeepSymbol::new(png_access_version_number as *mut ()),
    KeepSymbol::new(png_warning as *mut ()),
    KeepSymbol::new(png_get_IHDR as *mut ()),
    KeepSymbol::new(png_free as *mut ()),
];
