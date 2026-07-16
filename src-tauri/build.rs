fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let has_windres = std::process::Command::new("x86_64-w64-mingw32-windres")
            .arg("--version")
            .output()
            .is_ok()
            || std::process::Command::new("windres")
            .arg("--version")
            .output()
            .is_ok();
        if !has_windres {
            println!("cargo:warning=x86_64-w64-mingw32-windres not found. Bypassing Windows resource compilation for development cargo check.");
            return;
        }
    }
    tauri_build::build()
}
