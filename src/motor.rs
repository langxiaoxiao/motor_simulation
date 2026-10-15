use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DriveMode {
    FourBeat,  // 4拍全步进
    EightBeat, // 8拍半步进
}

pub struct StepperMotor {
    pub mode: DriveMode,
    pub is_running: bool,
    pub step_delay_ms: f32,
    pub current_step: usize,
    pub rotor_angle: f32,
    pub target_angle: f32,
    pub lag_angle: f32,          // 相位滞后角 Δθ
    last_step_time: Instant,

    // 线圈状态 (A相, B相极性: -1, 0, 1)
    pub phase_a_state: i32,
    pub phase_b_state: i32,
}

impl StepperMotor {
    // 8 拍半步进相序表 (角度, A相极性, B相极性, 描述文本)
    // 极性: 1 = 正向通电(N/A+), -1 = 反向通电(S/A-), 0 = 断电
    pub const STEPS_8: [(f32, i32, i32, &'static str); 8] = [
        (  0.0,  1,  0, "1: A+  B off"),
        ( 45.0,  1,  1, "2: A+  B+"),
        ( 90.0,  0,  1, "3: A off B+"),
        (135.0, -1,  1, "4: A-  B+"),
        (180.0, -1,  0, "5: A-  B off"),
        (225.0, -1, -1, "6: A-  B-"),
        (270.0,  0, -1, "7: A off B-"),
        (315.0,  1, -1, "8: A+  B-"),
    ];

    pub const STEPS_4: [(f32, i32, i32, &'static str); 4] = [
        (  0.0,  1,  1, "1: A+  B+"),
        ( 90.0, -1,  1, "2: A-  B+"),
        (180.0, -1, -1, "3: A-  B-"),
        (270.0,  1, -1, "4: A+  B-"),
    ];

    pub fn new() -> Self {
        Self {
            mode: DriveMode::EightBeat,
            is_running: false,
            step_delay_ms: 50.0,
            current_step: 0,
            rotor_angle: 0.0,
            target_angle: 0.0,
            lag_angle: 0.0,
            last_step_time: Instant::now(),
            phase_a_state: 1,
            phase_b_state: 0,
        }
    }

    /// 执行单步步进
    pub fn step_once(&mut self) {
        let max_steps = match self.mode {
            DriveMode::FourBeat => 4,
            DriveMode::EightBeat => 8,
        };

        self.current_step = (self.current_step + 1) % max_steps;
        self.apply_current_step();
        self.last_step_time = Instant::now();
    }

    pub fn apply_current_step(&mut self) {
        match self.mode {
            DriveMode::FourBeat => {
                let (angle, a, b, _) = Self::STEPS_4[self.current_step % 4];
                self.target_angle = angle;
                self.phase_a_state = a;
                self.phase_b_state = b;
            }
            DriveMode::EightBeat => {
                let (angle, a, b, _) = Self::STEPS_8[self.current_step % 8];
                self.target_angle = angle;
                self.phase_a_state = a;
                self.phase_b_state = b;
            }
        }
    }

    pub fn set_delay(&mut self, new_delay: f32) {
        self.step_delay_ms = new_delay.clamp(5.0, 500.0);
        self.last_step_time = Instant::now();
    }

    pub fn update(&mut self) {
        if self.is_running {
            let now = Instant::now();
            let interval = std::time::Duration::from_secs_f32(self.step_delay_ms / 1000.0);

            if now.duration_since(self.last_step_time) >= interval {
                self.step_once();
            }
        }

        // 转子平滑跟随磁场（带阻尼惯性）
        let mut diff = self.target_angle - self.rotor_angle;
        while diff > 180.0 { diff -= 360.0; }
        while diff < -180.0 { diff += 360.0; }

        // 动态跟随系数：如果速度极快产生滞后
        let responsiveness = if self.step_delay_ms < 15.0 { 0.15 } else { 0.35 };
        self.rotor_angle += diff * responsiveness;

        if self.rotor_angle < 0.0 { self.rotor_angle += 360.0; }
        if self.rotor_angle >= 360.0 { self.rotor_angle -= 360.0; }

        // 计算实时相位滞后角 Δθ
        let mut lag = (self.target_angle - self.rotor_angle).abs();
        if lag > 180.0 { lag = 360.0 - lag; }
        self.lag_angle = lag;
    }
}