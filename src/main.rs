mod app;
mod motor;
mod ui;

use app::StepperApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // 加载窗口图标的辅助函数
    let icon = load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 620.0])
        .with_resizable(true);

    // 如果成功读取图标，则设置到视口中
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Stepper Motor 2D Simulator (STM32 + DRV8335)",
        options,
        Box::new(|cc| {
            // ---- 中文字体加载逻辑保持不变 ----
            let mut fonts = egui::FontDefinitions::default();
            let font_path = if cfg!(target_os = "windows") {
                "C:\\Windows\\Fonts\\msyh.ttc"
            } else if cfg!(target_os = "macos") {
                "/System/Library/Fonts/PingFang.ttc"
            } else {
                "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"
            };

            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "custom_chinese_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "custom_chinese_font".to_owned());
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .push("custom_chinese_font".to_owned());
                cc.egui_ctx.set_fonts(fonts);
            }

            Ok(Box::new(StepperApp::new(cc)))
        }),
    )
}

/// 辅助函数：从文件加载图标并转换为 eframe::IconData
fn load_icon() -> Option<crate::egui::IconData> {
    // 假设你的图标放在项目根目录的 icon.png
    let image = image::open("icon.ico").ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}