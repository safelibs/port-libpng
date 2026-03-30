use std::process;

#[cold]
#[inline(never)]
pub(crate) fn placeholder_abort(symbol: &str) -> ! {
    eprintln!("libpng safe bootstrap stub invoked: {symbol}");
    process::abort();
}

include!(concat!(env!("OUT_DIR"), "/abi_export_stubs.rs"));
