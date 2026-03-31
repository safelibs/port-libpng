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

const LEGACY_RUNTIME_C_NAMES: &[&str] = &[
    "png",
    "pngerror",
    "pngget",
    "pngmem",
    "pngpread",
    "pngrio",
    "pngrtran",
    "pngrutil",
    "pngset",
    "pngtrans",
    "pngwrite",
    "pngwio",
    "pngwtran",
    "pngwutil",
];

const PNGREAD_SOURCE_NAME: &str = "pngread";

const UPSTREAM_RENAMES: &[(&str, &str)] = &[
    ("png_sig_cmp", "runtime_png_sig_cmp"),
    (
        "png_access_version_number",
        "runtime_png_access_version_number",
    ),
    ("png_get_libpng_ver", "runtime_png_get_libpng_ver"),
    ("png_get_header_ver", "runtime_png_get_header_ver"),
    ("png_get_header_version", "runtime_png_get_header_version"),
    ("png_get_copyright", "runtime_png_get_copyright"),
    (
        "png_build_grayscale_palette",
        "runtime_png_build_grayscale_palette",
    ),
    ("png_save_uint_32", "runtime_png_save_uint_32"),
    ("png_save_uint_16", "runtime_png_save_uint_16"),
    ("png_save_int_32", "runtime_png_save_int_32"),
    (
        "png_convert_to_rfc1123_buffer",
        "runtime_png_convert_to_rfc1123_buffer",
    ),
    (
        "png_convert_from_struct_tm",
        "runtime_png_convert_from_struct_tm",
    ),
    (
        "png_convert_from_time_t",
        "runtime_png_convert_from_time_t",
    ),
    ("png_set_sig_bytes", "runtime_png_set_sig_bytes"),
    (
        "png_get_compression_buffer_size",
        "runtime_png_get_compression_buffer_size",
    ),
    (
        "png_set_compression_buffer_size",
        "runtime_png_set_compression_buffer_size",
    ),
    ("png_create_read_struct", "runtime_png_create_read_struct"),
    (
        "png_create_write_struct",
        "runtime_png_create_write_struct",
    ),
    ("png_set_longjmp_fn", "runtime_png_set_longjmp_fn"),
    ("png_longjmp", "runtime_png_longjmp"),
    (
        "png_create_read_struct_2",
        "runtime_png_create_read_struct_2",
    ),
    (
        "png_create_write_struct_2",
        "runtime_png_create_write_struct_2",
    ),
    ("png_create_info_struct", "runtime_png_create_info_struct"),
    ("png_info_init_3", "runtime_png_info_init_3"),
    ("png_write_sig", "runtime_png_write_sig"),
    ("png_write_chunk", "runtime_png_write_chunk"),
    ("png_write_chunk_start", "runtime_png_write_chunk_start"),
    ("png_write_chunk_data", "runtime_png_write_chunk_data"),
    ("png_write_chunk_end", "runtime_png_write_chunk_end"),
    (
        "png_write_info_before_PLTE",
        "runtime_png_write_info_before_PLTE",
    ),
    ("png_write_info", "runtime_png_write_info"),
    (
        "png_destroy_info_struct",
        "runtime_png_destroy_info_struct",
    ),
    (
        "png_destroy_read_struct",
        "runtime_png_destroy_read_struct",
    ),
    (
        "png_destroy_write_struct",
        "runtime_png_destroy_write_struct",
    ),
    ("png_init_io", "runtime_png_init_io"),
    ("png_set_error_fn", "runtime_png_set_error_fn"),
    ("png_get_error_ptr", "runtime_png_get_error_ptr"),
    ("png_set_write_fn", "runtime_png_set_write_fn"),
    ("png_set_read_fn", "runtime_png_set_read_fn"),
    ("png_get_io_ptr", "runtime_png_get_io_ptr"),
    ("png_set_read_status_fn", "runtime_png_set_read_status_fn"),
    (
        "png_set_write_status_fn",
        "runtime_png_set_write_status_fn",
    ),
    ("png_set_mem_fn", "runtime_png_set_mem_fn"),
    ("png_get_mem_ptr", "runtime_png_get_mem_ptr"),
    (
        "png_set_read_user_transform_fn",
        "runtime_png_set_read_user_transform_fn",
    ),
    (
        "png_set_write_user_transform_fn",
        "runtime_png_set_write_user_transform_fn",
    ),
    (
        "png_set_user_transform_info",
        "runtime_png_set_user_transform_info",
    ),
    (
        "png_get_user_transform_ptr",
        "runtime_png_get_user_transform_ptr",
    ),
    (
        "png_set_read_user_chunk_fn",
        "runtime_png_set_read_user_chunk_fn",
    ),
    ("png_get_user_chunk_ptr", "runtime_png_get_user_chunk_ptr"),
    (
        "png_set_progressive_read_fn",
        "runtime_png_set_progressive_read_fn",
    ),
    (
        "png_get_progressive_ptr",
        "runtime_png_get_progressive_ptr",
    ),
    ("png_calloc", "runtime_png_calloc"),
    ("png_malloc", "runtime_png_malloc"),
    ("png_malloc_warn", "runtime_png_malloc_warn"),
    ("png_free", "runtime_png_free"),
    ("png_free_data", "runtime_png_free_data"),
    ("png_data_freer", "runtime_png_data_freer"),
    ("png_malloc_default", "runtime_png_malloc_default"),
    ("png_free_default", "runtime_png_free_default"),
    ("png_warning", "runtime_png_warning"),
    ("png_chunk_warning", "runtime_png_chunk_warning"),
    ("png_benign_error", "runtime_png_benign_error"),
    ("png_chunk_benign_error", "runtime_png_chunk_benign_error"),
    ("png_error", "runtime_png_error"),
    ("png_chunk_error", "runtime_png_chunk_error"),
    ("png_get_valid", "runtime_png_get_valid"),
    ("png_get_rowbytes", "runtime_png_get_rowbytes"),
    ("png_get_rows", "runtime_png_get_rows"),
    ("png_set_rows", "runtime_png_set_rows"),
    ("png_get_channels", "runtime_png_get_channels"),
    ("png_get_image_width", "runtime_png_get_image_width"),
    ("png_get_image_height", "runtime_png_get_image_height"),
    ("png_get_bit_depth", "runtime_png_get_bit_depth"),
    ("png_get_color_type", "runtime_png_get_color_type"),
    ("png_get_filter_type", "runtime_png_get_filter_type"),
    ("png_get_interlace_type", "runtime_png_get_interlace_type"),
    (
        "png_get_compression_type",
        "runtime_png_get_compression_type",
    ),
    ("png_set_benign_errors", "runtime_png_set_benign_errors"),
    ("png_get_user_width_max", "runtime_png_get_user_width_max"),
    (
        "png_get_user_height_max",
        "runtime_png_get_user_height_max",
    ),
    ("png_set_user_limits", "runtime_png_set_user_limits"),
    (
        "png_get_chunk_cache_max",
        "runtime_png_get_chunk_cache_max",
    ),
    (
        "png_set_chunk_cache_max",
        "runtime_png_set_chunk_cache_max",
    ),
    (
        "png_get_chunk_malloc_max",
        "runtime_png_get_chunk_malloc_max",
    ),
    (
        "png_set_chunk_malloc_max",
        "runtime_png_set_chunk_malloc_max",
    ),
    ("png_get_io_state", "runtime_png_get_io_state"),
    ("png_get_io_chunk_type", "runtime_png_get_io_chunk_type"),
    ("png_read_info", "runtime_png_read_info"),
    ("png_read_update_info", "runtime_png_read_update_info"),
    ("png_read_rows", "runtime_png_read_rows"),
    ("png_read_image", "runtime_png_read_image"),
    ("png_read_end", "runtime_png_read_end"),
    ("png_start_read_image", "runtime_png_start_read_image"),
    ("png_process_data", "runtime_png_process_data"),
    ("png_process_data_pause", "runtime_png_process_data_pause"),
    ("png_process_data_skip", "runtime_png_process_data_skip"),
    ("png_write_row", "runtime_png_write_row"),
    ("png_write_rows", "runtime_png_write_rows"),
    ("png_write_image", "runtime_png_write_image"),
    ("png_write_end", "runtime_png_write_end"),
    ("png_write_png", "runtime_png_write_png"),
    ("png_set_flush", "runtime_png_set_flush"),
    ("png_write_flush", "runtime_png_write_flush"),
    (
        "png_set_keep_unknown_chunks",
        "runtime_png_set_keep_unknown_chunks",
    ),
    (
        "png_set_check_for_invalid_index",
        "runtime_png_set_check_for_invalid_index",
    ),
    ("png_get_palette_max", "runtime_png_get_palette_max"),
    ("png_set_option", "runtime_png_set_option"),
    ("png_read_row", "runtime_png_read_row"),
    ("png_set_expand", "runtime_png_set_expand"),
    ("png_set_expand_16", "runtime_png_set_expand_16"),
    ("png_set_palette_to_rgb", "runtime_png_set_palette_to_rgb"),
    ("png_set_tRNS_to_alpha", "runtime_png_set_tRNS_to_alpha"),
    ("png_set_gray_to_rgb", "runtime_png_set_gray_to_rgb"),
    ("png_set_scale_16", "runtime_png_set_scale_16"),
    ("png_set_strip_16", "runtime_png_set_strip_16"),
    ("png_set_quantize", "runtime_png_set_quantize"),
    ("png_set_shift", "runtime_png_set_shift"),
    ("png_set_swap", "runtime_png_set_swap"),
    ("png_set_swap_alpha", "runtime_png_set_swap_alpha"),
    ("png_set_invert_alpha", "runtime_png_set_invert_alpha"),
    ("png_set_invert_mono", "runtime_png_set_invert_mono"),
    ("png_set_bgr", "runtime_png_set_bgr"),
    ("png_set_filler", "runtime_png_set_filler"),
    ("png_set_add_alpha", "runtime_png_set_add_alpha"),
    ("png_set_packing", "runtime_png_set_packing"),
    ("png_set_packswap", "runtime_png_set_packswap"),
    ("png_set_filter", "runtime_png_set_filter"),
    (
        "png_set_filter_heuristics",
        "runtime_png_set_filter_heuristics",
    ),
    (
        "png_set_filter_heuristics_fixed",
        "runtime_png_set_filter_heuristics_fixed",
    ),
    (
        "png_set_compression_level",
        "runtime_png_set_compression_level",
    ),
    (
        "png_set_compression_mem_level",
        "runtime_png_set_compression_mem_level",
    ),
    (
        "png_set_compression_method",
        "runtime_png_set_compression_method",
    ),
    (
        "png_set_compression_strategy",
        "runtime_png_set_compression_strategy",
    ),
    (
        "png_set_compression_window_bits",
        "runtime_png_set_compression_window_bits",
    ),
    (
        "png_set_text_compression_level",
        "runtime_png_set_text_compression_level",
    ),
    (
        "png_set_text_compression_mem_level",
        "runtime_png_set_text_compression_mem_level",
    ),
    (
        "png_set_text_compression_method",
        "runtime_png_set_text_compression_method",
    ),
    (
        "png_set_text_compression_strategy",
        "runtime_png_set_text_compression_strategy",
    ),
    (
        "png_set_text_compression_window_bits",
        "runtime_png_set_text_compression_window_bits",
    ),
    (
        "png_set_interlace_handling",
        "runtime_png_set_interlace_handling",
    ),
    ("png_set_rgb_to_gray", "runtime_png_set_rgb_to_gray"),
    (
        "png_set_rgb_to_gray_fixed",
        "runtime_png_set_rgb_to_gray_fixed",
    ),
    ("png_set_background", "runtime_png_set_background"),
    (
        "png_set_background_fixed",
        "runtime_png_set_background_fixed",
    ),
    ("png_set_alpha_mode", "runtime_png_set_alpha_mode"),
    (
        "png_set_alpha_mode_fixed",
        "runtime_png_set_alpha_mode_fixed",
    ),
    ("png_get_bKGD", "runtime_png_get_bKGD"),
    ("png_set_bKGD", "runtime_png_set_bKGD"),
    ("png_get_cHRM", "runtime_png_get_cHRM"),
    ("png_get_cHRM_fixed", "runtime_png_get_cHRM_fixed"),
    ("png_set_cHRM", "runtime_png_set_cHRM"),
    ("png_set_cHRM_fixed", "runtime_png_set_cHRM_fixed"),
    ("png_set_cHRM_XYZ", "runtime_png_set_cHRM_XYZ"),
    ("png_set_cHRM_XYZ_fixed", "runtime_png_set_cHRM_XYZ_fixed"),
    ("png_get_gAMA", "runtime_png_get_gAMA"),
    ("png_get_gAMA_fixed", "runtime_png_get_gAMA_fixed"),
    ("png_set_gAMA", "runtime_png_set_gAMA"),
    ("png_set_gAMA_fixed", "runtime_png_set_gAMA_fixed"),
    ("png_get_hIST", "runtime_png_get_hIST"),
    ("png_set_hIST", "runtime_png_set_hIST"),
    ("png_get_IHDR", "runtime_png_get_IHDR"),
    ("png_set_IHDR", "runtime_png_set_IHDR"),
    ("png_get_oFFs", "runtime_png_get_oFFs"),
    ("png_set_oFFs", "runtime_png_set_oFFs"),
    ("png_get_pCAL", "runtime_png_get_pCAL"),
    ("png_set_pCAL", "runtime_png_set_pCAL"),
    ("png_get_pHYs", "runtime_png_get_pHYs"),
    ("png_set_pHYs", "runtime_png_set_pHYs"),
    ("png_get_PLTE", "runtime_png_get_PLTE"),
    ("png_set_PLTE", "runtime_png_set_PLTE"),
    ("png_get_sBIT", "runtime_png_get_sBIT"),
    ("png_set_sBIT", "runtime_png_set_sBIT"),
    ("png_get_sRGB", "runtime_png_get_sRGB"),
    ("png_set_sRGB", "runtime_png_set_sRGB"),
    (
        "png_set_sRGB_gAMA_and_cHRM",
        "runtime_png_set_sRGB_gAMA_and_cHRM",
    ),
    ("png_get_iCCP", "runtime_png_get_iCCP"),
    ("png_set_iCCP", "runtime_png_set_iCCP"),
    ("png_get_sPLT", "runtime_png_get_sPLT"),
    ("png_set_sPLT", "runtime_png_set_sPLT"),
    ("png_get_text", "runtime_png_get_text"),
    ("png_set_text", "runtime_png_set_text"),
    ("png_get_tIME", "runtime_png_get_tIME"),
    ("png_set_tIME", "runtime_png_set_tIME"),
    ("png_get_tRNS", "runtime_png_get_tRNS"),
    ("png_set_tRNS", "runtime_png_set_tRNS"),
    ("png_get_sCAL", "runtime_png_get_sCAL"),
    ("png_get_sCAL_fixed", "runtime_png_get_sCAL_fixed"),
    ("png_get_sCAL_s", "runtime_png_get_sCAL_s"),
    ("png_set_sCAL", "runtime_png_set_sCAL"),
    ("png_set_sCAL_fixed", "runtime_png_set_sCAL_fixed"),
    ("png_set_sCAL_s", "runtime_png_set_sCAL_s"),
    ("png_get_eXIf", "runtime_png_get_eXIf"),
    ("png_set_eXIf", "runtime_png_set_eXIf"),
    ("png_get_eXIf_1", "runtime_png_get_eXIf_1"),
    ("png_set_eXIf_1", "runtime_png_set_eXIf_1"),
    ("png_get_cHRM_XYZ", "runtime_png_get_cHRM_XYZ"),
    ("png_get_cHRM_XYZ_fixed", "runtime_png_get_cHRM_XYZ_fixed"),
    (
        "png_image_write_to_file",
        "runtime_png_image_write_to_file",
    ),
    (
        "png_image_write_to_stdio",
        "runtime_png_image_write_to_stdio",
    ),
    (
        "png_image_write_to_memory",
        "runtime_png_image_write_to_memory",
    ),
    ("png_image_free", "runtime_png_image_free"),
];

const PNGREAD_DEFINITION_RENAMES: &[(&str, &str)] = &[
    ("png_create_read_struct", "runtime_png_create_read_struct"),
    (
        "png_create_read_struct_2",
        "runtime_png_create_read_struct_2",
    ),
    ("png_read_info", "runtime_png_read_info"),
    ("png_read_update_info", "runtime_png_read_update_info"),
    ("png_start_read_image", "runtime_png_start_read_image"),
    ("png_read_row", "runtime_png_read_row"),
    ("png_read_rows", "runtime_png_read_rows"),
    ("png_read_image", "runtime_png_read_image"),
    ("png_read_end", "runtime_png_read_end"),
    (
        "png_destroy_read_struct",
        "runtime_png_destroy_read_struct",
    ),
    ("png_set_read_status_fn", "runtime_png_set_read_status_fn"),
    (
        "png_image_begin_read_from_file",
        "runtime_png_image_begin_read_from_file",
    ),
    (
        "png_image_begin_read_from_stdio",
        "runtime_png_image_begin_read_from_stdio",
    ),
    (
        "png_image_begin_read_from_memory",
        "runtime_png_image_begin_read_from_memory",
    ),
    ("png_image_finish_read", "runtime_png_image_finish_read"),
];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let profile = env::var("PROFILE")?;
    let target = env::var("TARGET")?;
    let original_root = original_tree_dir(&manifest_dir);

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
        cshim_dir.join("read_phase_bridge.c"),
        cshim_dir.join("longjmp_bridge.c"),
        manifest_dir.join("src/lib.rs"),
        manifest_dir.join("src/abi_exports.rs"),
        manifest_dir.join("src/common.rs"),
        manifest_dir.join("src/error.rs"),
        manifest_dir.join("src/get.rs"),
        manifest_dir.join("src/io.rs"),
        manifest_dir.join("src/memory.rs"),
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
        original_root.join("png.h"),
        original_root.join("pngconf.h"),
        original_root.join("pngpriv.h"),
        original_root.join("pngstruct.h"),
        original_root.join("pnginfo.h"),
        original_c_source(&manifest_dir, PNGREAD_SOURCE_NAME),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    for stem in LEGACY_RUNTIME_C_NAMES {
        println!(
            "cargo:rerun-if-changed={}",
            original_c_source(&manifest_dir, stem).display()
        );
    }

    let generated_stubs = out_dir.join("abi_export_stubs.rs");
    fs::write(
        &generated_stubs,
        "// Mixed Rust/upstream libpng build; no generated Rust export stubs required.\n",
    )?;

    cc::Build::new()
        .file(cshim_dir.join("read_phase_bridge.c"))
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(&original_root)
        .compile("png16_read_phase_bridge");

    cc::Build::new()
        .file(cshim_dir.join("longjmp_bridge.c"))
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(&original_root)
        .compile("png16_longjmp_bridge");

    let adapted_pngread = out_dir.join("pngread_rust_entry_bridge.c");
    generate_pngread_bridge_source(
        &original_c_source(&manifest_dir, PNGREAD_SOURCE_NAME),
        &adapted_pngread,
    )?;

    let mut upstream = cc::Build::new();
    upstream
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(&original_root)
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

    for stem in LEGACY_RUNTIME_C_NAMES {
        upstream.file(original_c_source(&manifest_dir, stem));
    }

    upstream.compile("png16_upstream");

    cc::Build::new()
        .file(&adapted_pngread)
        .warnings(true)
        .std("c99")
        .include(&include_dir)
        .include(&original_root)
        .define("PNG_DISABLE_ADLER32_CHECK_SUPPORTED", "1")
        .define("PNG_INTEL_SSE_OPT", "0")
        .define("PNG_ARM_NEON_OPT", "0")
        .define("PNG_MIPS_MMI_OPT", "0")
        .define("PNG_MIPS_MSA_OPT", "0")
        .define("PNG_POWERPC_VSX_OPT", "0")
        .define("PNG_LOONGARCH_LSX_OPT", "0")
        .compile("png16_pngread_bridge");

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

fn original_tree_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("..").join("original")
}

fn original_c_source(manifest_dir: &Path, stem: &str) -> PathBuf {
    original_tree_dir(manifest_dir).join(format!("{stem}.c"))
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

fn generate_pngread_bridge_source(
    original_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut source = fs::read_to_string(original_path)?;

    for &(symbol, renamed) in PNGREAD_DEFINITION_RENAMES {
        rename_pngread_definition(&mut source, symbol, renamed)?;
    }

    fs::write(output_path, source)?;
    Ok(())
}

fn rename_pngread_definition(
    source: &mut String,
    symbol: &str,
    renamed: &str,
) -> Result<(), Box<dyn Error>> {
    let line_start = format!("\n{symbol}(");
    let line_start_replacement = format!("\n{renamed}(");
    if replace_unique(source, &line_start, &line_start_replacement)? {
        return Ok(());
    }

    let inline_pngapi = format!(" PNGAPI {symbol}(");
    let inline_pngapi_replacement = format!(" PNGAPI {renamed}(");
    if replace_unique(source, &inline_pngapi, &inline_pngapi_replacement)? {
        return Ok(());
    }

    let macro_form = format!("\n{symbol},(");
    let macro_form_replacement = format!("\n{renamed},(");
    if replace_unique(source, &macro_form, &macro_form_replacement)? {
        return Ok(());
    }

    Err(format!("failed to rewrite pngread.c definition for symbol {symbol}").into())
}

fn replace_unique(
    source: &mut String,
    needle: &str,
    replacement: &str,
) -> Result<bool, Box<dyn Error>> {
    let matches = source.matches(needle).count();
    if matches == 0 {
        return Ok(false);
    }

    if matches != 1 {
        return Err(
            format!("expected exactly one occurrence of {needle:?}, found {matches}").into(),
        );
    }

    *source = source.replacen(needle, replacement, 1);
    Ok(true)
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
