use crate::game::RGB;
use core::time::Duration;
use libm::{fabs, sin};
use core::f64::consts::PI;

pub struct Animation {
    state: usize,
    last_update_time: Duration,
    animation_speed: Duration,
}

impl Animation {
    pub fn new(animation_speed: Duration) -> Self {
        Self {
            state: 0,
            last_update_time: Duration::from_millis(0),
            animation_speed,
        }
    }

    pub fn update(&mut self, current_time: Duration) -> bool {
        if current_time - self.last_update_time > self.animation_speed {
            self.last_update_time = current_time;

            if self.state >= 20 {
                self.state = 0;
                return false;
            } else {
                self.state += 1;
                return true;
            }
        }
        true
    }

    pub fn get_color(&self) -> RGB {
        let f: f64 = self.state as f64;
        let s = fabs(sin(f * 2.0 * PI / 20.0)) * 10.0 + 10.0;
        let color = (s as u8 * 10, s as u8 * 10, s as u8 * 10);
        RGB::new(color.0, color.1, color.2)
    }
}
