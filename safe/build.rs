use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

const ABI_BASENAME: &str = "png16";
const LIBPNG_VERSION: &str = "1.6.43";
const FULL_SO_NAME: &str = "libpng16.so.16.43.0";
const SONAME: &str = "libpng16.so.16";
const LIBS_PRIVATE: &str = "-lm -lz -lm ";
const IMPLEMENTED_EXPORTS: &[&str] = &[
    "png_access_version_number",
    "png_benign_error",
    "png_build_grayscale_palette",
    "png_calloc",
    "png_chunk_benign_error",
    "png_chunk_error",
    "png_chunk_warning",
    "png_convert_from_struct_tm",
    "png_convert_from_time_t",
    "png_convert_to_rfc1123",
    "png_convert_to_rfc1123_buffer",
    "png_create_info_struct",
    "png_create_read_struct",
    "png_create_read_struct_2",
    "png_create_write_struct",
    "png_create_write_struct_2",
    "png_data_freer",
    "png_destroy_info_struct",
    "png_destroy_read_struct",
    "png_destroy_write_struct",
    "png_error",
    "png_free",
    "png_free_data",
    "png_free_default",
    "png_get_bit_depth",
    "png_get_channels",
    "png_get_chunk_cache_max",
    "png_get_chunk_malloc_max",
    "png_get_color_type",
    "png_get_compression_type",
    "png_get_copyright",
    "png_get_error_ptr",
    "png_get_filter_type",
    "png_get_header_ver",
    "png_get_header_version",
    "png_get_image_height",
    "png_get_image_width",
    "png_get_int_32",
    "png_get_interlace_type",
    "png_get_io_chunk_type",
    "png_get_io_ptr",
    "png_get_io_state",
    "png_get_libpng_ver",
    "png_get_mem_ptr",
    "png_get_palette_max",
    "png_get_progressive_ptr",
    "png_get_rowbytes",
    "png_get_rows",
    "png_get_uint_16",
    "png_get_uint_31",
    "png_get_uint_32",
    "png_get_user_chunk_ptr",
    "png_get_user_height_max",
    "png_get_user_transform_ptr",
    "png_get_user_width_max",
    "png_get_valid",
    "png_info_init_3",
    "png_init_io",
    "png_longjmp",
    "png_malloc",
    "png_malloc_default",
    "png_malloc_warn",
    "png_save_int_32",
    "png_save_uint_16",
    "png_save_uint_32",
    "png_set_benign_errors",
    "png_set_check_for_invalid_index",
    "png_set_chunk_cache_max",
    "png_set_chunk_malloc_max",
    "png_set_error_fn",
    "png_set_longjmp_fn",
    "png_set_mem_fn",
    "png_set_option",
    "png_set_progressive_read_fn",
    "png_set_read_fn",
    "png_set_read_status_fn",
    "png_set_read_user_chunk_fn",
    "png_set_read_user_transform_fn",
    "png_set_rows",
    "png_set_sig_bytes",
    "png_set_user_limits",
    "png_set_user_transform_info",
    "png_set_write_fn",
    "png_set_write_status_fn",
    "png_set_write_user_transform_fn",
    "png_sig_cmp",
    "png_warning",
];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let profile = env::var("PROFILE")?;
    let target = env::var("TARGET")?;

    let include_dir = manifest_dir.join("include");
    let abi_dir = manifest_dir.join("abi");
    let pkg_dir = manifest_dir.join("pkg");
    let cshim_dir = manifest_dir.join("cshim");
    let version_script = abi_dir.join("libpng.vers");
    let exports_file = abi_dir.join("exports.txt");

    for path in [
        manifest_dir.join("build.rs"),
        include_dir.join("png.h"),
        include_dir.join("pngconf.h"),
        include_dir.join("pnglibconf.h"),
        version_script.clone(),
        exports_file.clone(),
        pkg_dir.join("libpng.pc.in"),
        pkg_dir.join("libpng-config.in"),
        cshim_dir.join("longjmp_bridge.c"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/abi_exports.rs"),
        manifest_dir.join("src/common.rs"),
        manifest_dir.join("src/error.rs"),
        manifest_dir.join("src/get.rs"),
        manifest_dir.join("src/io.rs"),
        manifest_dir.join("src/memory.rs"),
        manifest_dir.join("src/set.rs"),
        manifest_dir.join("src/state.rs"),
        manifest_dir.join("src/types.rs"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let generated_stubs = out_dir.join("abi_export_stubs.rs");
    generate_stub_module(&exports_file, &generated_stubs)?;

    cc::Build::new()
        .file(cshim_dir.join("longjmp_bridge.c"))
        .warnings(true)
        .compile("png16_longjmp_bridge");

    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        version_script.display()
    );
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,{SONAME}");

    let profile_dir = profile_dir_from_out_dir(&out_dir, &profile)?;
    let stage_root = profile_dir.join("abi-stage");
    let multiarch = detect_multiarch(&target);
    stage_install_tree(&manifest_dir, &profile_dir, &stage_root, &multiarch)?;

    Ok(())
}

fn generate_stub_module(exports_file: &Path, output_file: &Path) -> Result<(), Box<dyn Error>> {
    let exports = fs::read_to_string(exports_file)?;
    let mut seen = BTreeSet::new();
    let mut generated = String::from("// @generated by build.rs\n\n");

    for export in exports
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if IMPLEMENTED_EXPORTS.contains(&export) {
            continue;
        }

        if !seen.insert(export.to_owned()) {
            return Err(format!("duplicate export in {}: {export}", exports_file.display()).into());
        }

        generated.push_str("#[unsafe(no_mangle)]\n");
        generated.push_str(&format!("pub extern \"C\" fn {export}() {{\n"));
        generated.push_str(&format!("    placeholder_abort(\"{export}\");\n"));
        generated.push_str("}\n\n");
    }

    fs::write(output_file, generated)?;
    Ok(())
}

fn stage_install_tree(
    manifest_dir: &Path,
    profile_dir: &Path,
    stage_root: &Path,
    multiarch: &str,
) -> Result<(), Box<dyn Error>> {
    if stage_root.exists() {
        fs::remove_dir_all(stage_root)?;
    }

    let include_root = stage_root.join("usr/include");
    let include_subdir = include_root.join("libpng16");
    let lib_root = stage_root.join("usr/lib").join(multiarch);
    let pkg_root = lib_root.join("pkgconfig");
    let bin_root = stage_root.join("usr/bin");

    fs::create_dir_all(&include_subdir)?;
    fs::create_dir_all(&pkg_root)?;
    fs::create_dir_all(&bin_root)?;

    for header in ["png.h", "pngconf.h", "pnglibconf.h"] {
        fs::copy(
            manifest_dir.join("include").join(header),
            include_subdir.join(header),
        )?;
        ensure_symlink(
            include_root.join(header),
            Path::new("libpng16").join(header),
        )?;
    }

    let libdir = format!("/usr/lib/{multiarch}");
    let rendered_pc = render_template(
        &manifest_dir.join("pkg/libpng.pc.in"),
        &[
            ("@prefix@", "/usr"),
            ("@exec_prefix@", "${prefix}"),
            ("@libdir@", &libdir),
            ("@includedir@", "/usr/include"),
            ("@PNGLIB_MAJOR@", "1"),
            ("@PNGLIB_MINOR@", "6"),
            ("@PNGLIB_VERSION@", LIBPNG_VERSION),
            ("@LIBS@", LIBS_PRIVATE),
        ],
    )?;
    fs::write(pkg_root.join("libpng16.pc"), rendered_pc)?;
    ensure_symlink(pkg_root.join("libpng.pc"), Path::new("libpng16.pc"))?;

    let rendered_config = render_template(
        &manifest_dir.join("pkg/libpng-config.in"),
        &[
            ("@prefix@", "/usr"),
            ("@exec_prefix@", "${prefix}"),
            ("@libdir@", &libdir),
            ("@includedir@", "/usr/include"),
            ("@PNGLIB_MAJOR@", "1"),
            ("@PNGLIB_MINOR@", "6"),
            ("@PNGLIB_VERSION@", LIBPNG_VERSION),
            ("@LIBS@", LIBS_PRIVATE),
        ],
    )?;
    write_executable(bin_root.join("libpng16-config"), &rendered_config)?;
    ensure_symlink(bin_root.join("libpng-config"), Path::new("libpng16-config"))?;

    ensure_symlink(
        lib_root.join(FULL_SO_NAME),
        profile_dir.join(format!("lib{ABI_BASENAME}.so")),
    )?;
    ensure_symlink(lib_root.join("libpng16.so.16"), Path::new(FULL_SO_NAME))?;
    ensure_symlink(lib_root.join("libpng16.so"), Path::new(FULL_SO_NAME))?;
    ensure_symlink(lib_root.join("libpng.so"), Path::new("libpng16.so"))?;
    ensure_symlink(
        lib_root.join("libpng16.a"),
        profile_dir.join(format!("lib{ABI_BASENAME}.a")),
    )?;
    ensure_symlink(lib_root.join("libpng.a"), Path::new("libpng16.a"))?;

    Ok(())
}

fn render_template(
    template: &Path,
    replacements: &[(&str, &str)],
) -> Result<String, Box<dyn Error>> {
    let mut rendered = fs::read_to_string(template)?;

    for (needle, value) in replacements {
        rendered = rendered.replace(needle, value);
    }

    Ok(rendered)
}

fn profile_dir_from_out_dir(out_dir: &Path, profile: &str) -> Result<PathBuf, Box<dyn Error>> {
    for ancestor in out_dir.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some(profile) {
            return Ok(ancestor.to_path_buf());
        }
    }

    Err(format!(
        "unable to locate profile directory {profile} from OUT_DIR {}",
        out_dir.display()
    )
    .into())
}

fn detect_multiarch(target: &str) -> String {
    if let Ok(value) = env::var("LIBPNG_MULTIARCH") {
        if !value.trim().is_empty() {
            return value;
        }
    }

    for (program, args) in [
        ("dpkg-architecture", vec!["-qDEB_HOST_MULTIARCH"]),
        ("gcc", vec!["-print-multiarch"]),
    ] {
        if let Ok(output) = Command::new(program).args(args).output() {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                if !value.is_empty() {
                    return value;
                }
            }
        }
    }

    let parts: Vec<&str> = target.split('-').collect();
    if parts.len() >= 4 && parts[parts.len() - 2] == "linux" && parts[parts.len() - 1] == "gnu" {
        return format!("{}-linux-gnu", parts[0]);
    }

    target.to_owned()
}

fn ensure_symlink(path: PathBuf, target: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    symlink(target.as_ref(), path)?;
    Ok(())
}

fn write_executable(path: PathBuf, contents: &str) -> Result<(), Box<dyn Error>> {
    fs::write(&path, contents)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}
