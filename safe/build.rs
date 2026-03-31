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
        cshim_dir.join("read_phase_bridge.c"),
        cshim_dir.join("longjmp_bridge.c"),
    ] {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        version_script.display()
    );
    println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,{SONAME}");

    let profile_dir = profile_dir_from_out_dir(&out_dir, &profile)?;
    let static_output = profile_dir.join(format!("lib{ABI_BASENAME}.a"));

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
