pub(crate) const ABI_EXPORTS_TEXT: &str = include_str!("../abi/exports.txt");

#[allow(dead_code)]
pub(crate) fn exported_symbols() -> impl Iterator<Item = &'static str> {
    ABI_EXPORTS_TEXT
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}
