use std::error::Error;

// The final `impl_remove_upstream_c_runtime` phase no longer synthesizes or
// compiles any hidden libpng support runtime from vendor C sources. The file
// remains checked in only to preserve the existing artifact path.
pub(crate) fn build_support_core() -> Result<(), Box<dyn Error>> {
    Ok(())
}
