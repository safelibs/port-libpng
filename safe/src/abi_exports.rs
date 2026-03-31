include!(concat!(env!("OUT_DIR"), "/abi_export_stubs.rs"));

#[allow(dead_code)]
pub(crate) fn exported_symbols() -> &'static [&'static str] {
    ABI_EXPORTS
}
