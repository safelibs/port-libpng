use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const INTERNAL_SYMBOL_PREFIX: &str = "bridge_";
const SUPPORT_BUILD_LABEL: &str = "png16_support_core";

fn load_symbol_list(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut symbols = Vec::new();
    for line in fs::read_to_string(path)?.lines() {
        let symbol = line.trim();
        if symbol.is_empty() || symbol.starts_with('#') {
            continue;
        }
        symbols.push(symbol.to_owned());
    }
    Ok(symbols)
}

fn rust_owned_exports(src_dir: &Path, abi_exports: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let abi_set: std::collections::HashSet<&str> = abi_exports.iter().map(String::as_str).collect();
    let mut owned = Vec::new();

    for entry in fs::read_dir(src_dir)? {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }

        for line in fs::read_to_string(&path)?.lines() {
            let Some(fn_pos) = line.find("extern \"C\" fn ") else {
                continue;
            };
            let name_start = fn_pos + "extern \"C\" fn ".len();
            let name = line[name_start..]
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .collect::<String>();
            if name.starts_with("png_")
                && abi_set.contains(name.as_str())
                && !owned.iter().any(|existing: &String| existing == &name)
            {
                owned.push(name);
            }
        }
    }

    owned.sort();
    Ok(owned)
}

fn vendor_tree_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..").join("original")
}

fn support_sources(vendor_root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut sources: Vec<PathBuf> = fs::read_dir(vendor_root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "c")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("png") && name != "pngtest.c")
        })
        .collect();
    sources.sort();
    Ok(sources)
}

fn write_internal_symbol_header(out_dir: &Path, exports: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    let header_path = out_dir.join("support_core_symbols.h");
    let mut header = String::from("/* generated internal libpng symbol remapping */\n");
    for symbol in exports {
        header.push_str("#define ");
        header.push_str(symbol);
        header.push(' ');
        header.push_str(INTERNAL_SYMBOL_PREFIX);
        header.push_str(symbol);
        header.push('\n');
    }
    fs::write(&header_path, header)?;
    Ok(header_path)
}

fn write_support_wrappers(out_dir: &Path, sources: &[PathBuf], rename_header: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut wrappers = Vec::new();
    for source in sources {
        let stem = source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("invalid support source {}", source.display()))?;
        let wrapper = out_dir.join(format!("{stem}_support_wrapper.c"));
        let source_body = fs::read_to_string(source)?;
        let wrapper_body = format!(
            "/* generated support translation unit */\n#include \"{}\"\n{}\n",
            rename_header.display(),
            source_body,
        );
        fs::write(&wrapper, wrapper_body)?;
        wrappers.push(wrapper);
    }
    Ok(wrappers)
}

fn write_abi_export_stub_manifest(out_dir: &Path, exports: &[String]) -> Result<(), Box<dyn Error>> {
    let stubs = out_dir.join("abi_export_stubs.rs");
    let mut body = String::from("pub(crate) const ABI_EXPORTS: &[&str] = &[\n");
    for symbol in exports {
        body.push_str("    \"");
        body.push_str(symbol);
        body.push_str("\",\n");
    }
    body.push_str("];\n");
    fs::write(&stubs, body)?;
    Ok(())
}

pub(crate) fn build_support_core(
    manifest_dir: &Path,
    out_dir: &Path,
    include_dir: &Path,
    exports_file: &Path,
) -> Result<(), Box<dyn Error>> {
    let support_dir = manifest_dir.join("tools/build_support");
    let export_aliases_file = support_dir.join("export_aliases.txt");
    let vendor_root = vendor_tree_dir(manifest_dir);
    let exports = load_symbol_list(exports_file)?;
    let mut rust_exports = load_symbol_list(&export_aliases_file)?;
    for symbol in rust_owned_exports(&manifest_dir.join("src"), &exports)? {
        if !rust_exports.iter().any(|existing| existing == &symbol) {
            rust_exports.push(symbol);
        }
    }
    rust_exports.sort();

    for path in [
        export_aliases_file,
        support_dir.join("read_support_impl.inc.c"),
        support_dir.join("longjmp_support_impl.inc.c"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/abi_exports.rs"),
        manifest_dir.join("src/common.rs"),
        manifest_dir.join("src/error.rs"),
        manifest_dir.join("src/get.rs"),
        manifest_dir.join("src/io.rs"),
        manifest_dir.join("src/memory.rs"),
        manifest_dir.join("src/bridge_ffi.rs"),
        manifest_dir.join("src/read.rs"),
        manifest_dir.join("src/read_progressive.rs"),
        manifest_dir.join("src/read_transform.rs"),
        manifest_dir.join("src/read_util.rs"),
        manifest_dir.join("src/colorspace.rs"),
        manifest_dir.join("src/set.rs"),
        manifest_dir.join("src/simplified.rs"),
        manifest_dir.join("src/state.rs"),
        manifest_dir.join("src/chunks.rs"),
        manifest_dir.join("src/interlace.rs"),
        manifest_dir.join("src/types.rs"),
        manifest_dir.join("src/write.rs"),
        manifest_dir.join("src/write_transform.rs"),
        manifest_dir.join("src/write_util.rs"),
        manifest_dir.join("src/zlib.rs"),
        vendor_root.join("png.h"),
        vendor_root.join("pngconf.h"),
        vendor_root.join("pngpriv.h"),
        vendor_root.join("pngstruct.h"),
        vendor_root.join("pnginfo.h"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let support_sources = support_sources(&vendor_root)?;
    for source in &support_sources {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    write_abi_export_stub_manifest(out_dir, &rust_exports)?;
    let rename_header = write_internal_symbol_header(out_dir, &rust_exports)?;
    let support_wrappers = write_support_wrappers(out_dir, &support_sources, &rename_header)?;

    cc::Build::new()
        .file(support_dir.join("read_support_impl.inc.c"))
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(&vendor_root)
        .compile("png16_read_support");

    cc::Build::new()
        .file(support_dir.join("longjmp_support_impl.inc.c"))
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(&vendor_root)
        .compile("png16_longjmp_support");

    let mut support_core = cc::Build::new();
    support_core
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(&vendor_root)
        .include(out_dir)
        .define("PNG_DISABLE_ADLER32_CHECK_SUPPORTED", "1")
        .define("PNG_INTEL_SSE_OPT", "0")
        .define("PNG_ARM_NEON_OPT", "0")
        .define("PNG_MIPS_MMI_OPT", "0")
        .define("PNG_MIPS_MSA_OPT", "0")
        .define("PNG_POWERPC_VSX_OPT", "0")
        .define("PNG_LOONGARCH_LSX_OPT", "0");

    for wrapper in &support_wrappers {
        support_core.file(wrapper);
    }

    support_core.compile(SUPPORT_BUILD_LABEL);

    Ok(())
}
