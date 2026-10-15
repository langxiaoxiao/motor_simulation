use eframe::egui;
use crate::motor::{StepperMotor, DriveMode};

pub struct MotorUi;

impl MotorUi {
    pub fn render(ui: &mut egui::Ui, motor: &mut StepperMotor) {
        let ctx = ui.ctx();

        // 样式微调：设置现代科技风圆角与视觉间距
        let mut style = (*ctx.style()).clone();
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(255, 255, 255);
        ctx.set_style(style);

        // 顶栏说明标题
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("⚡ 两相四线双极性定子磁场与永磁转子同步响应模拟器")
                    .strong()
                    .size(14.0)
                    .color(egui::Color32::from_rgb(30, 41, 59))
            );
        });
        ui.add_space(6.0);

        // ==================== 1. 顶部 4 个状态信息卡片 ====================
        let current_desc = match motor.mode {
            DriveMode::FourBeat => StepperMotor::STEPS_4[motor.current_step % 4].3,
            DriveMode::EightBeat => StepperMotor::STEPS_8[motor.current_step % 8].3,
        };

        ui.columns(4, |columns| {
            status_card(&mut columns[0], "当前相位", current_desc, egui::Color32::from_rgb(241, 245, 249));
            status_card(&mut columns[1], "磁场角度 B", &format!("{:.1}°", motor.target_angle), egui::Color32::from_rgb(241, 245, 249));
            status_card(&mut columns[2], "转子位置 θ", &format!("{:.1}°", motor.rotor_angle), egui::Color32::from_rgb(241, 245, 249));
            status_card(&mut columns[3], "相位滞后角 Δθ", &format!("{:.1}°", motor.lag_angle), egui::Color32::from_rgb(241, 245, 249));
        });

        ui.add_space(8.0);

        // ==================== 2. 节拍快速点击切换条（自动换行，防溢出） ====================
        ui.horizontal_wrapped(|ui| {
            let max_steps = match motor.mode {
                DriveMode::FourBeat => 4,
                DriveMode::EightBeat => 8,
            };

            for i in 0..max_steps {
                let text = match motor.mode {
                    DriveMode::FourBeat => format!("拍 {}: {}", i + 1, StepperMotor::STEPS_4[i].3),
                    DriveMode::EightBeat => format!("拍 {}: {}", i + 1, StepperMotor::STEPS_8[i].3),
                };

                let is_active = motor.current_step == i;
                let btn = egui::Button::new(text).selected(is_active);
                if ui.add(btn).clicked() {
                    motor.current_step = i;
                    motor.apply_current_step();
                }
            }

            // 模式切换单选钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_8beat = motor.mode == DriveMode::EightBeat;
                if ui.radio(is_8beat, "8拍半步").clicked() && !is_8beat {
                    motor.mode = DriveMode::EightBeat;
                    motor.current_step = 0;
                    motor.apply_current_step();
                }
                if ui.radio(!is_8beat, "4拍全步").clicked() && is_8beat {
                    motor.mode = DriveMode::FourBeat;
                    motor.current_step = 0;
                    motor.apply_current_step();
                }
            });
        });

        ui.separator();

        // ==================== 3. 中间 2D 磁场与转子交互 Canvas ====================
        let (response, painter) = ui.allocate_painter(egui::vec2(ui.available_width(), 300.0), egui::Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let radius = 100.0;

        // 浅灰背景圆形定子大盘
        painter.circle_filled(center, radius + 22.0, egui::Color32::from_rgb(248, 250, 252));
        painter.circle_stroke(center, radius + 22.0, egui::Stroke::new(1.5, egui::Color32::from_rgb(226, 232, 240)));

        // 定义四相线圈坐标 (A+, A-, B+, B-)
        let coils = [
            ("B+", center + egui::vec2(0.0, -radius), motor.phase_b_state > 0, motor.phase_b_state < 0),
            ("A+", center + egui::vec2(radius, 0.0), motor.phase_a_state > 0, motor.phase_a_state < 0),
            ("B-", center + egui::vec2(0.0, radius), motor.phase_b_state < 0, motor.phase_b_state > 0),
            ("A-", center + egui::vec2(-radius, 0.0), motor.phase_a_state < 0, motor.phase_a_state > 0),
        ];

        for (name, pos, is_active_n, is_active_s) in coils {
            let bg_color = if is_active_n {
                egui::Color32::from_rgb(37, 99, 235) // 活跃 N 极蓝
            } else if is_active_s {
                egui::Color32::from_rgb(219, 39, 119) // 活跃 S 极粉
            } else {
                egui::Color32::from_rgb(226, 232, 240) // 未通电灰
            };

            let text_color = if is_active_n || is_active_s {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(100, 116, 139)
            };

            painter.rect_filled(
                egui::Rect::from_center_size(pos, egui::vec2(46.0, 32.0)),
                6.0,
                bg_color,
            );

            let pole_text = if is_active_n { "N" } else if is_active_s { "S" } else { "-" };

            painter.text(
                pos + egui::vec2(0.0, -6.0),
                egui::Align2::CENTER_CENTER,
                pole_text,
                egui::FontId::proportional(13.0),
                text_color,
            );
            painter.text(
                pos + egui::vec2(0.0, 8.0),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(11.0),
                text_color,
            );
        }

        // ==================== 永磁转子精准渲染 ====================
        let rotor_rad = motor.rotor_angle.to_radians();
        let rotor_radius = 52.0;

        // 转子底层深色科技风圆盘
        painter.circle_filled(center, rotor_radius, egui::Color32::from_rgb(15, 23, 42));
        painter.circle_stroke(center, rotor_radius, egui::Stroke::new(2.0, egui::Color32::from_rgb(71, 85, 105)));

        // 旋转直径分隔线，划开 N/S 极区域
        let perp_rad = rotor_rad + std::f32::consts::FRAC_PI_2;
        let p1 = center + egui::vec2(perp_rad.cos(), perp_rad.sin()) * rotor_radius;
        let p2 = center - egui::vec2(perp_rad.cos(), perp_rad.sin()) * rotor_radius;
        painter.line_segment([p1, p2], egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 65, 85)));

        // N 极与 S 极文字对称定位
        let dir_x = rotor_rad.cos();
        let dir_y = rotor_rad.sin();
        let dir_vec = egui::vec2(dir_x, dir_y);

        let n_pos = center + dir_vec * (rotor_radius * 0.5);
        let s_pos = center - dir_vec * (rotor_radius * 0.5);

        painter.text(n_pos, egui::Align2::CENTER_CENTER, "N", egui::FontId::proportional(16.0), egui::Color32::from_rgb(96, 165, 250));
        painter.text(s_pos, egui::Align2::CENTER_CENTER, "S", egui::FontId::proportional(16.0), egui::Color32::from_rgb(244, 114, 182));

        // 转子中心白色轴心
        painter.circle_filled(center, 6.0, egui::Color32::WHITE);
        painter.circle_stroke(center, 6.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

        // 转子边缘随角度同步旋转的绿色指示小圆点
        let edge_dot_pos = center + dir_vec * rotor_radius;
        painter.circle_filled(edge_dot_pos, 4.5, egui::Color32::from_rgb(34, 197, 94));

        // 外部绿色目标合成磁场指示小圆点
        let target_rad = motor.target_angle.to_radians();
        let target_pos = center + egui::vec2(target_rad.cos(), target_rad.sin()) * (radius - 12.0);
        painter.circle_filled(target_pos, 5.0, egui::Color32::from_rgb(34, 197, 94));

        ui.separator();

        // ==================== 4. 底部控制栏与图例说明 ====================
        ui.horizontal(|ui| {
            ui.label("脉冲换相延时:");
            let mut delay = motor.step_delay_ms;
            let slider_response = ui.add(
                egui::Slider::new(&mut delay, 5.0..=500.0)
                    .suffix("ms")
                    .drag_value_speed(5.0)
            );
            if slider_response.changed() {
                motor.set_delay(delay);
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let run_btn_text = if motor.is_running { "⏸ 暂停仿真" } else { "▶ 启动仿真" };
            if ui.button(run_btn_text).clicked() {
                motor.is_running = !motor.is_running;
            }

            if ui.button("单步执行").clicked() {
                motor.is_running = false;
                motor.step_once();
            }

            if ui.button("复位归零").clicked() {
                motor.is_running = false;
                motor.rotor_angle = 0.0;
                motor.current_step = 0;
                motor.apply_current_step();
            }

            // 右侧图例指示
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(34, 197, 94), "● 合成磁场 B");
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::from_rgb(219, 39, 119), "● S极/通电");
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::from_rgb(37, 99, 235), "● N极/通电");
                });
            });
        });
    }
}

/// 辅助函数：绘制顶部状态卡片
fn status_card(ui: &mut egui::Ui, title: &str, value: &str, bg_color: egui::Color32) {
    let size = egui::vec2(ui.available_width(), 50.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

    ui.painter().rect_filled(rect, 6.0, bg_color);
    ui.painter().rect_stroke(rect, 6.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(226, 232, 240)));

    let inner_rect = rect.shrink(8.0);
    ui.painter().text(
        inner_rect.left_top() + egui::vec2(0.0, 2.0),
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(11.0),
        egui::Color32::from_rgb(100, 116, 139),
    );
    ui.painter().text(
        inner_rect.left_bottom() + egui::vec2(0.0, -2.0),
        egui::Align2::LEFT_BOTTOM,
        value,
        egui::FontId::proportional(14.0),
        egui::Color32::from_rgb(15, 23, 42),
    );
}