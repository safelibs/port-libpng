use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

const INTERNAL_SYMBOL_PREFIX: &str = "bridge_";
const SUPPORT_BUILD_LABEL: &str = "png16_internal_support";
const PAYLOAD_MAGIC: &[u8] = b"LPNGPACK1\0";

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

fn read_u32(cursor: &mut &[u8]) -> Result<u32, Box<dyn Error>> {
    if cursor.len() < 4 {
        return Err("payload truncated while reading u32".into());
    }
    let (value, rest) = cursor.split_at(4);
    *cursor = rest;
    Ok(u32::from_le_bytes(value.try_into()?))
}

fn read_u64(cursor: &mut &[u8]) -> Result<u64, Box<dyn Error>> {
    if cursor.len() < 8 {
        return Err("payload truncated while reading u64".into());
    }
    let (value, rest) = cursor.split_at(8);
    *cursor = rest;
    Ok(u64::from_le_bytes(value.try_into()?))
}

fn unpack_payload(payload_file: &Path, out_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let payload = fs::read(payload_file)?;
    let mut cursor = payload.as_slice();

    if !cursor.starts_with(PAYLOAD_MAGIC) {
        return Err(format!("unsupported payload header in {}", payload_file.display()).into());
    }
    cursor = &cursor[PAYLOAD_MAGIC.len()..];

    let entry_count = read_u32(&mut cursor)?;
    let payload_root = out_dir.join("payload");

    if payload_root.exists() {
        fs::remove_dir_all(&payload_root)?;
    }
    fs::create_dir_all(&payload_root)?;

    for _ in 0..entry_count {
        let path_len = usize::try_from(read_u32(&mut cursor)?)?;
        if cursor.len() < path_len {
            return Err("payload truncated while reading entry path".into());
        }
        let (path_bytes, rest) = cursor.split_at(path_len);
        cursor = rest;

        let data_len = usize::try_from(read_u64(&mut cursor)?)?;
        if cursor.len() < data_len {
            return Err("payload truncated while reading entry body".into());
        }
        let (data, rest) = cursor.split_at(data_len);
        cursor = rest;

        let relpath = std::str::from_utf8(path_bytes)?;
        let output = out_dir.join(relpath);
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, data)?;
    }

    Ok(payload_root)
}

fn support_sources(payload_root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut sources: Vec<PathBuf> = fs::read_dir(payload_root.join("internal"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "c"))
        .collect();
    sources.sort();
    Ok(sources)
}

pub(crate) fn build_support_core(
    manifest_dir: &Path,
    out_dir: &Path,
    include_dir: &Path,
    exports_file: &Path,
) -> Result<(), Box<dyn Error>> {
    let support_dir = manifest_dir.join("tools/build_support");
    let payload_file = support_dir.join("compat_payload.bin");
    let exports = load_symbol_list(exports_file)?;

    for path in [
        payload_file.clone(),
        support_dir.join("build_support.rs"),
        manifest_dir.join("src/bridge_ffi.rs"),
        manifest_dir.join("src/abi_exports.rs"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/memory.rs"),
        manifest_dir.join("src/state.rs"),
        manifest_dir.join("src/write.rs"),
        manifest_dir.join("src/write_transform.rs"),
        manifest_dir.join("src/write_util.rs"),
        manifest_dir.join("src/simplified.rs"),
        manifest_dir.join("src/common.rs"),
        include_dir.join("png.h"),
        include_dir.join("pngconf.h"),
        include_dir.join("pnglibconf.h"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let payload_root = unpack_payload(&payload_file, out_dir)?;
    let mut renamed_exports = load_symbol_list(&payload_root.join("export_aliases.txt"))?;
    for symbol in rust_owned_exports(&manifest_dir.join("src"), &exports)? {
        if !renamed_exports.iter().any(|existing| existing == &symbol) {
            renamed_exports.push(symbol);
        }
    }
    renamed_exports.sort();
    let support_sources = support_sources(&payload_root)?;
    let rename_header = write_internal_symbol_header(out_dir, &renamed_exports)?;
    println!("cargo:rerun-if-changed={}", rename_header.display());

    cc::Build::new()
        .file(payload_root.join("read_support_impl.c"))
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(payload_root.join("include"))
        .include(out_dir)
        .compile("png16_read_support");

    cc::Build::new()
        .file(payload_root.join("longjmp_support_impl.c"))
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(payload_root.join("include"))
        .include(out_dir)
        .compile("png16_longjmp_support");

    let mut support_core = cc::Build::new();
    support_core
        .warnings(true)
        .std("c99")
        .include(include_dir)
        .include(payload_root.join("include"))
        .include(out_dir)
        .define("PNG_DISABLE_ADLER32_CHECK_SUPPORTED", "1")
        .define("PNG_INTEL_SSE_OPT", "0")
        .define("PNG_ARM_NEON_OPT", "0")
        .define("PNG_MIPS_MMI_OPT", "0")
        .define("PNG_MIPS_MSA_OPT", "0")
        .define("PNG_POWERPC_VSX_OPT", "0")
        .define("PNG_LOONGARCH_LSX_OPT", "0");

    for source in &support_sources {
        support_core.file(source);
    }

    support_core.compile(SUPPORT_BUILD_LABEL);
    Ok(())
}
