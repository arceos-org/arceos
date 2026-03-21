// Simulated Hardware Abstraction Layer (HAL).
//
// In a real deployment these functions would talk to actual hardware peripherals
// via GPIO, I2C, SPI, etc. Here they print actions and return simulated results.

use log::info;

use crate::base64::encode_base64;

// ---------------------------------------------------------------------------
// HAL state
// ---------------------------------------------------------------------------

pub struct HalInterface {
    /// Simulated position (x, y) in cm from origin.
    pos_x: f64,
    pos_y: f64,
    /// Heading in degrees (0 = north, 90 = east).
    heading: f64,
    /// Battery percentage.
    battery_pct: f32,
    /// Mechanical arm state.
    arm_lowered: bool,
    gripper_closed: bool,
    /// Photo counter — cycles through available images.
    photo_index: usize,
    /// Boot instant for uptime tracking.
    boot_time: std::time::Instant,
}

impl HalInterface {
    pub fn init() -> Self {
        info!("[hal] Hardware abstraction layer initialized (simulated mode)");
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            heading: 0.0,
            battery_pct: 100.0,
            arm_lowered: false,
            gripper_closed: false,
            photo_index: 0,
            boot_time: std::time::Instant::now(),
        }
    }

    pub fn move_forward(&mut self, distance_cm: f64) -> String {
        let rad = self.heading.to_radians();
        self.pos_x += distance_cm * rad.sin();
        self.pos_y += distance_cm * rad.cos();
        self.battery_pct = (self.battery_pct - 0.5).max(0.0);
        format!(
            "已向前移动 {:.0}cm。当前位置: ({:.0}, {:.0}), 朝向 {:.0}°",
            distance_cm, self.pos_x, self.pos_y, self.heading
        )
    }

    pub fn move_backward(&mut self, distance_cm: f64) -> String {
        let rad = self.heading.to_radians();
        self.pos_x -= distance_cm * rad.sin();
        self.pos_y -= distance_cm * rad.cos();
        self.battery_pct = (self.battery_pct - 0.5).max(0.0);
        format!(
            "已向后移动 {:.0}cm。当前位置: ({:.0}, {:.0}), 朝向 {:.0}°",
            distance_cm, self.pos_x, self.pos_y, self.heading
        )
    }

    pub fn turn(&mut self, direction: &str, angle_deg: f64) -> String {
        match direction {
            "left" => self.heading = (self.heading - angle_deg) % 360.0,
            "right" => self.heading = (self.heading + angle_deg) % 360.0,
            _ => return format!("未知方向: {}", direction),
        }
        if self.heading < 0.0 {
            self.heading += 360.0;
        }
        self.battery_pct = (self.battery_pct - 0.2).max(0.0);
        format!(
            "已向{}转 {:.0}°。当前朝向: {:.0}°",
            if direction == "left" { "左" } else { "右" },
            angle_deg,
            self.heading
        )
    }

    pub fn stop(&mut self) -> String {
        "已停止所有运动。".to_string()
    }

    pub fn arm_lower(&mut self) -> String {
        if self.arm_lowered {
            return "机械臂已经在工作位置。".to_string();
        }
        self.arm_lowered = true;
        self.battery_pct = (self.battery_pct - 0.3).max(0.0);
        "机械臂已放下到工作位置。".to_string()
    }

    pub fn arm_raise(&mut self) -> String {
        if !self.arm_lowered {
            return "机械臂已经在收纳位置。".to_string();
        }
        self.arm_lowered = false;
        self.battery_pct = (self.battery_pct - 0.3).max(0.0);
        "机械臂已抬起到收纳位置。".to_string()
    }

    pub fn gripper_close(&mut self) -> String {
        if self.gripper_closed {
            return "夹子已经是闭合状态。".to_string();
        }
        self.gripper_closed = true;
        "夹子已收紧，物体已抓取。".to_string()
    }

    pub fn gripper_open(&mut self) -> String {
        if !self.gripper_closed {
            return "夹子已经是张开状态。".to_string();
        }
        self.gripper_closed = false;
        "夹子已张开，物体已释放。".to_string()
    }

    /// Take a photo: read an image file and return (description, base64_data).
    /// Cycles through ball_1.png .. ball_15.png, ball_null.png.
    pub fn take_photo(&mut self) -> (String, String) {
        let filenames = [
            "ball_1.png",
            "ball_2.png",
            "ball_3.png",
            "ball_4.png",
            "ball_5.png",
            "ball_6.png",
            "ball_7.png",
            "ball_8.png",
            "ball_9.png",
            "ball_10.png",
            "ball_11.png",
            "ball_12.png",
            "ball_13.png",
            "ball_14.png",
            "ball_15.png",
            "ball_null.png",
        ];

        let idx = self.photo_index % filenames.len();
        self.photo_index += 1;
        let filename = filenames[idx];
        let path = format!("/bw_bls/{}", filename);

        eprintln!("[hal] Taking photo: reading {}", path);

        match std::fs::read(&path) {
            Ok(data) => {
                let b64 = encode_base64(&data);
                eprintln!(
                    "[hal] Photo captured: {} ({} bytes -> {} b64 chars)",
                    filename,
                    data.len(),
                    b64.len()
                );
                let desc = format!(
                    "已拍摄照片（来自 {}）。图像数据已附加到对话中，请查看并描述你看到的内容。",
                    filename
                );
                (desc, b64)
            }
            Err(e) => {
                eprintln!("[hal] Failed to read {}: {}", path, e);
                let desc = format!("拍照失败: 无法读取文件 {} ({})", path, e);
                // Return empty image on failure
                (desc, String::new())
            }
        }
    }

    pub fn get_time(&self) -> String {
        let uptime = self.boot_time.elapsed();
        format!(
            "系统运行时间: {:.1} 秒（注意：ArceOS 没有 RTC，无法获取真实时间）",
            uptime.as_secs_f64()
        )
    }

    pub fn get_status(&self) -> String {
        format!(
            "位置: ({:.0}, {:.0})cm, 朝向: {:.0}°, 电池: {:.0}%, 机械臂: {}, 夹子: {}",
            self.pos_x,
            self.pos_y,
            self.heading,
            self.battery_pct,
            if self.arm_lowered {
                "工作位置"
            } else {
                "收纳位置"
            },
            if self.gripper_closed {
                "闭合"
            } else {
                "张开"
            }
        )
    }
}
