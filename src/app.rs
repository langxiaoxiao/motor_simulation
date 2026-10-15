use eframe::egui;
use crate::motor::StepperMotor;
use crate::ui::MotorUi;

pub struct StepperApp {
    motor: StepperMotor,
}

impl StepperApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            motor: StepperMotor::new(),
        }
    }
}

impl eframe::App for StepperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 更新物理状态
        self.motor.update();

        // 渲染主面板
        egui::CentralPanel::default().show(ctx, |ui| {
            MotorUi::render(ui, &mut self.motor);
        });

        // 持续请求重绘，保证动画流畅
        ctx.request_repaint();
    }
}