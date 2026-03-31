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

const UPSTREAM_SOURCES: &[&str] = &[
    "../original/png.c",
    "../original/pngerror.c",
    "../original/pngget.c",
    "../original/pngmem.c",
    "../original/pngpread.c",
    "../original/pngread.c",
    "../original/pngrio.c",
    "../original/pngrtran.c",
    "../original/pngrutil.c",
    "../original/pngset.c",
    "../original/pngtrans.c",
    "../original/pngwrite.c",
    "../original/pngwio.c",
    "../original/pngwtran.c",
    "../original/pngwutil.c",
];

const UPSTREAM_RENAMES: &[(&str, &str)] = &[
    ("png_sig_cmp", "upstream_png_sig_cmp"),
    ("png_access_version_number", "upstream_png_access_version_number"),
    ("png_get_libpng_ver", "upstream_png_get_libpng_ver"),
    ("png_get_header_ver", "upstream_png_get_header_ver"),
    ("png_get_header_version", "upstream_png_get_header_version"),
    ("png_get_copyright", "upstream_png_get_copyright"),
    ("png_build_grayscale_palette", "upstream_png_build_grayscale_palette"),
    ("png_get_uint_31", "upstream_png_get_uint_31"),
    ("png_save_uint_32", "upstream_png_save_uint_32"),
    ("png_save_uint_16", "upstream_png_save_uint_16"),
    ("png_save_int_32", "upstream_png_save_int_32"),
    (
        "png_convert_to_rfc1123_buffer",
        "upstream_png_convert_to_rfc1123_buffer",
    ),
    ("png_convert_to_rfc1123", "upstream_png_convert_to_rfc1123"),
    ("png_convert_from_struct_tm", "upstream_png_convert_from_struct_tm"),
    ("png_convert_from_time_t", "upstream_png_convert_from_time_t"),
    ("png_warning", "upstream_png_warning"),
    ("png_error", "upstream_png_error"),
    ("png_benign_error", "upstream_png_benign_error"),
    ("png_chunk_warning", "upstream_png_chunk_warning"),
    ("png_chunk_error", "upstream_png_chunk_error"),
    ("png_chunk_benign_error", "upstream_png_chunk_benign_error"),
    ("png_set_error_fn", "upstream_png_set_error_fn"),
    ("png_get_error_ptr", "upstream_png_get_error_ptr"),
    ("png_set_longjmp_fn", "upstream_png_set_longjmp_fn"),
    ("png_longjmp", "upstream_png_longjmp"),
    ("png_calloc", "upstream_png_calloc"),
    ("png_malloc", "upstream_png_malloc"),
    ("png_malloc_default", "upstream_png_malloc_default"),
    ("png_malloc_warn", "upstream_png_malloc_warn"),
    ("png_free", "upstream_png_free"),
    ("png_free_default", "upstream_png_free_default"),
    ("png_set_mem_fn", "upstream_png_set_mem_fn"),
    ("png_get_mem_ptr", "upstream_png_get_mem_ptr"),
    ("png_create_read_struct", "upstream_png_create_read_struct"),
    ("png_create_read_struct_2", "upstream_png_create_read_struct_2"),
    ("png_create_write_struct", "upstream_png_create_write_struct"),
    ("png_create_write_struct_2", "upstream_png_create_write_struct_2"),
    ("png_create_info_struct", "upstream_png_create_info_struct"),
    ("png_destroy_info_struct", "upstream_png_destroy_info_struct"),
    ("png_info_init_3", "upstream_png_info_init_3"),
    ("png_data_freer", "upstream_png_data_freer"),
    ("png_free_data", "upstream_png_free_data"),
    ("png_destroy_read_struct", "upstream_png_destroy_read_struct"),
    ("png_destroy_write_struct", "upstream_png_destroy_write_struct"),
    ("png_get_io_ptr", "upstream_png_get_io_ptr"),
    ("png_init_io", "upstream_png_init_io"),
    ("png_set_read_fn", "upstream_png_set_read_fn"),
    ("png_set_write_fn", "upstream_png_set_write_fn"),
    ("png_set_read_status_fn", "upstream_png_set_read_status_fn"),
    ("png_set_write_status_fn", "upstream_png_set_write_status_fn"),
    (
        "png_set_progressive_read_fn",
        "upstream_png_set_progressive_read_fn",
    ),
    ("png_get_progressive_ptr", "upstream_png_get_progressive_ptr"),
    (
        "png_set_read_user_chunk_fn",
        "upstream_png_set_read_user_chunk_fn",
    ),
    ("png_get_user_chunk_ptr", "upstream_png_get_user_chunk_ptr"),
    (
        "png_set_read_user_transform_fn",
        "upstream_png_set_read_user_transform_fn",
    ),
    (
        "png_set_write_user_transform_fn",
        "upstream_png_set_write_user_transform_fn",
    ),
    (
        "png_set_user_transform_info",
        "upstream_png_set_user_transform_info",
    ),
    (
        "png_get_user_transform_ptr",
        "upstream_png_get_user_transform_ptr",
    ),
    ("png_get_io_state", "upstream_png_get_io_state"),
    ("png_get_io_chunk_type", "upstream_png_get_io_chunk_type"),
    ("png_get_valid", "upstream_png_get_valid"),
    ("png_get_rowbytes", "upstream_png_get_rowbytes"),
    ("png_get_rows", "upstream_png_get_rows"),
    ("png_get_image_width", "upstream_png_get_image_width"),
    ("png_get_image_height", "upstream_png_get_image_height"),
    ("png_get_bit_depth", "upstream_png_get_bit_depth"),
    ("png_get_color_type", "upstream_png_get_color_type"),
    ("png_get_filter_type", "upstream_png_get_filter_type"),
    ("png_get_interlace_type", "upstream_png_get_interlace_type"),
    ("png_get_compression_type", "upstream_png_get_compression_type"),
    ("png_get_channels", "upstream_png_get_channels"),
    ("png_get_user_width_max", "upstream_png_get_user_width_max"),
    ("png_get_user_height_max", "upstream_png_get_user_height_max"),
    ("png_get_chunk_cache_max", "upstream_png_get_chunk_cache_max"),
    (
        "png_get_chunk_malloc_max",
        "upstream_png_get_chunk_malloc_max",
    ),
    ("png_get_palette_max", "upstream_png_get_palette_max"),
    ("png_set_sig_bytes", "upstream_png_set_sig_bytes"),
    ("png_set_rows", "upstream_png_set_rows"),
    ("png_set_user_limits", "upstream_png_set_user_limits"),
    ("png_set_chunk_cache_max", "upstream_png_set_chunk_cache_max"),
    (
        "png_set_chunk_malloc_max",
        "upstream_png_set_chunk_malloc_max",
    ),
    ("png_set_benign_errors", "upstream_png_set_benign_errors"),
    (
        "png_set_check_for_invalid_index",
        "upstream_png_set_check_for_invalid_index",
    ),
    ("png_set_option", "upstream_png_set_option"),
    ("png_read_row", "upstream_png_read_row"),
    ("png_set_expand", "upstream_png_set_expand"),
    ("png_set_expand_16", "upstream_png_set_expand_16"),
    ("png_set_palette_to_rgb", "upstream_png_set_palette_to_rgb"),
    ("png_set_tRNS_to_alpha", "upstream_png_set_tRNS_to_alpha"),
    ("png_set_gray_to_rgb", "upstream_png_set_gray_to_rgb"),
    ("png_set_scale_16", "upstream_png_set_scale_16"),
    ("png_set_strip_16", "upstream_png_set_strip_16"),
    ("png_set_quantize", "upstream_png_set_quantize"),
    ("png_set_shift", "upstream_png_set_shift"),
    ("png_set_swap_alpha", "upstream_png_set_swap_alpha"),
    ("png_set_invert_alpha", "upstream_png_set_invert_alpha"),
    ("png_set_invert_mono", "upstream_png_set_invert_mono"),
    ("png_set_bgr", "upstream_png_set_bgr"),
    (
        "png_set_interlace_handling",
        "upstream_png_set_interlace_handling",
    ),
    ("png_set_rgb_to_gray", "upstream_png_set_rgb_to_gray"),
    (
        "png_set_rgb_to_gray_fixed",
        "upstream_png_set_rgb_to_gray_fixed",
    ),
    ("png_set_background", "upstream_png_set_background"),
    (
        "png_set_background_fixed",
        "upstream_png_set_background_fixed",
    ),
    ("png_set_alpha_mode", "upstream_png_set_alpha_mode"),
    (
        "png_set_alpha_mode_fixed",
        "upstream_png_set_alpha_mode_fixed",
    ),
    ("png_set_cHRM_XYZ", "upstream_png_set_cHRM_XYZ"),
    ("png_set_cHRM_XYZ_fixed", "upstream_png_set_cHRM_XYZ_fixed"),
    ("png_get_cHRM_XYZ", "upstream_png_get_cHRM_XYZ"),
    ("png_get_cHRM_XYZ_fixed", "upstream_png_get_cHRM_XYZ_fixed"),
    (
        "png_image_begin_read_from_file",
        "upstream_png_image_begin_read_from_file",
    ),
    (
        "png_image_begin_read_from_stdio",
        "upstream_png_image_begin_read_from_stdio",
    ),
    (
        "png_image_begin_read_from_memory",
        "upstream_png_image_begin_read_from_memory",
    ),
    ("png_image_finish_read", "upstream_png_image_finish_read"),
    ("png_image_free", "upstream_png_image_free"),
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
        exports_file,
        pkg_dir.join("libpng.pc.in"),
        pkg_dir.join("libpng-config.in"),
        cshim_dir.join("longjmp_bridge.c"),
        cshim_dir.join("read_phase_bridge.c"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/abi_exports.rs"),
        manifest_dir.join("src/common.rs"),
        manifest_dir.join("src/error.rs"),
        manifest_dir.join("src/get.rs"),
        manifest_dir.join("src/io.rs"),
        manifest_dir.join("src/memory.rs"),
        manifest_dir.join("src/read_progressive.rs"),
        manifest_dir.join("src/read_transform.rs"),
        manifest_dir.join("src/read_util.rs"),
        manifest_dir.join("src/colorspace.rs"),
        manifest_dir.join("src/set.rs"),
        manifest_dir.join("src/simplified.rs"),
        manifest_dir.join("src/state.rs"),
        manifest_dir.join("src/chunks.rs"),
        manifest_dir.join("src/interlace.rs"),
        manifest_dir.join("src/zlib.rs"),
        manifest_dir.join("../original/png.h"),
        manifest_dir.join("../original/pngconf.h"),
        manifest_dir.join("../original/pngpriv.h"),
        manifest_dir.join("../original/pngstruct.h"),
        manifest_dir.join("../original/pnginfo.h"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    for source in UPSTREAM_SOURCES {
        println!(
            "cargo:rerun-if-changed={}",
            manifest_dir.join(source).display()
        );
    }

    let generated_stubs = out_dir.join("abi_export_stubs.rs");
    fs::write(
        &generated_stubs,
        "// Mixed Rust/upstream libpng build; no generated Rust export stubs required.\n",
    )?;

    cc::Build::new()
        .file(cshim_dir.join("longjmp_bridge.c"))
        .warnings(true)
        .compile("png16_longjmp_bridge");

    cc::Build::new()
        .file(cshim_dir.join("read_phase_bridge.c"))
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(manifest_dir.join("../original"))
        .compile("png16_read_phase_bridge");

    let mut upstream = cc::Build::new();
    upstream
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(manifest_dir.join("../original"))
        .define("PNG_DISABLE_ADLER32_CHECK_SUPPORTED", "1")
        .define("PNG_INTEL_SSE_OPT", "0")
        .define("PNG_ARM_NEON_OPT", "0")
        .define("PNG_MIPS_MMI_OPT", "0")
        .define("PNG_MIPS_MSA_OPT", "0")
        .define("PNG_POWERPC_VSX_OPT", "0")
        .define("PNG_LOONGARCH_LSX_OPT", "0");

    for &(symbol, renamed) in UPSTREAM_RENAMES {
        upstream.define(symbol, renamed);
    }

    for source in UPSTREAM_SOURCES {
        upstream.file(manifest_dir.join(source));
    }

    upstream.compile("png16_upstream");

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        version_script.display()
    );
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,{SONAME}");

    let profile_dir = profile_dir_from_out_dir(&out_dir, &profile)?;
    let static_output = profile_dir.join(format!("lib{ABI_BASENAME}.a"));

    // On a clean build these artifacts do not exist until after rustc finishes
    // linking the library, so the staged install tree must be populated by a
    // post-build helper rather than unconditionally from the build script.
    if static_output.exists() {
        let stage_root = profile_dir.join("abi-stage");
        let multiarch = detect_multiarch(&target);
        stage_install_tree(&manifest_dir, &profile_dir, &stage_root, &multiarch)?;
    }

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
    ensure_symlink(include_root.join("libpng"), Path::new("libpng16"))?;

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

    link_versioned_shared_library(
        manifest_dir,
        &profile_dir.join(format!("lib{ABI_BASENAME}.a")),
        &lib_root.join(FULL_SO_NAME),
    )?;
    ensure_symlink(lib_root.join("libpng16.so.16"), Path::new(FULL_SO_NAME))?;
    ensure_symlink(lib_root.join("libpng16.so"), Path::new(FULL_SO_NAME))?;
    ensure_symlink(lib_root.join("libpng.so"), Path::new("libpng16.so"))?;
    fs::copy(
        profile_dir.join(format!("lib{ABI_BASENAME}.a")),
        lib_root.join("libpng16.a"),
    )?;
    ensure_symlink(lib_root.join("libpng.a"), Path::new("libpng16.a"))?;

    Ok(())
}

fn link_versioned_shared_library(
    manifest_dir: &Path,
    static_lib: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let compiler = cc::Build::new().get_compiler();
    let mut command = compiler.to_command();
    command
        .arg("-shared")
        .arg("-Wl,--whole-archive")
        .arg(static_lib)
        .arg("-Wl,--no-whole-archive")
        .arg(format!(
            "-Wl,--version-script={}",
            manifest_dir.join("abi/libpng.vers").display()
        ))
        .arg(format!("-Wl,-soname,{SONAME}"))
        .arg("-lz")
        .arg("-lm")
        .arg("-ldl")
        .arg("-lpthread")
        .arg("-lrt")
        .arg("-lutil")
        .arg("-lgcc_s")
        .arg("-o")
        .arg(output_path);

    let status = command.status()?;
    if !status.success() {
        return Err(format!(
            "failed to link staged shared library {} from {}",
            output_path.display(),
            static_lib.display()
        )
        .into());
    }

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

fn ensure_symlink(link_path: PathBuf, target: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    if link_path.exists() || link_path.symlink_metadata().is_ok() {
        fs::remove_file(&link_path)?;
    }
    symlink(target.as_ref(), link_path)?;
    Ok(())
}

fn write_executable(path: PathBuf, contents: &str) -> Result<(), Box<dyn Error>> {
    fs::write(&path, contents)?;
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}
