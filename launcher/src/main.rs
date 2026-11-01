use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const APP_DIR: &str = "CrealityPrint-7.3.0.AppDir";
const LOCALE_FIX: &str = "pt_BR.UTF-8";
const NVIDIA_GLX_LIB: &str = "libGLX_nvidia.so.0";

fn main() {
    let exe_dir = env::current_exe()
        .expect("não foi possível determinar o caminho do executável")
        .parent()
        .expect("executável sem diretório pai")
        .to_path_buf();

    let app_dir = exe_dir.join(APP_DIR);
    let binary = app_dir.join("bin").join("CrealityPrint");

    if !binary.is_file() {
        eprintln!(
            "crealityprint-rs: binário não encontrado em {}",
            binary.display()
        );
        std::process::exit(1);
    }

    let lib_path = format!(
        "{}:{}",
        app_dir.join("bin").display(),
        app_dir.join("usr/lib").display()
);
    let ld_library_path = match env::var("LD_LIBRARY_PATH") {
        Ok(existing) if !existing.is_empty() => format!("{lib_path}:{existing}"),
        _ => lib_path,
    };

    if env::var_os("__GLX_VENDOR_LIBRARY_NAME").is_none() && has_nvidia_glx() {
        if boot_vga_is_not_nvidia() {
            env::set_var("__GLX_VENDOR_LIBRARY_NAME", "mesa");
        }
    }

    let args: Vec<String> = env::args().skip(1).collect();

    let err = Command::new(&binary)
        .args(&args)
        .env("LD_LIBRARY_PATH", ld_library_path)
        .env("LC_ALL", LOCALE_FIX)
        .exec();

    eprintln!("crealityprint-rs: falha ao executar {}: {err}", binary.display());
    std::process::exit(1);
}

fn has_nvidia_glx() -> bool {
    for ldconfig in ["ldconfig", "/usr/sbin/ldconfig", "/sbin/ldconfig"] {
        if let Ok(output) = Command::new(ldconfig).arg("-p").output() {
            if String::from_utf8_lossy(&output.stdout).contains(NVIDIA_GLX_LIB) {
                return true;
            }
            break;
        }
    }

    for base in ["/usr/lib", "/lib"] {
        let base = Path::new(base);
        if base.join(NVIDIA_GLX_LIB).exists() {
            return true;
        }
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                if entry.path().join(NVIDIA_GLX_LIB).exists() {
                    return true;
                }
            }
        }
    }
    false
}

fn boot_vga_is_not_nvidia() -> bool {
    let drm = PathBuf::from("/sys/class/drm");
    let Ok(entries) = fs::read_dir(&drm) else {
        return false;
    };

    entries.flatten().any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") {
            return false;
        }
        let device = entry.path().join("device");
        let boot_vga = fs::read_to_string(device.join("boot_vga")).unwrap_or_default();
        let vendor = fs::read_to_string(device.join("vendor")).unwrap_or_default();
        boot_vga.trim() == "1" && vendor.trim() != "0x10de"
    })
}
