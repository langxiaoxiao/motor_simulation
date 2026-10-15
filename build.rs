// build.rs
fn main() {
    // 仅在 Windows 编译目标下生效
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "target_os = windows" || cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // 指定你的 .ico 图标文件路径
        res.set_icon("icon.ico");

        if let Err(e) = res.compile() {
            eprintln!("Failed to set Windows exe icon: {}", e);
            std::process::exit(1);
        }
    }
}