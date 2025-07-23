use std::time::{Duration, SystemTime};

pub struct FpsManager {
    last_frame: SystemTime,
    delta: f32,
}

impl FpsManager {
    pub fn register_frame(&mut self) {
        let now = SystemTime::now();
        let duration = now.duration_since(self.last_frame);

        let duration: Duration =
            duration.unwrap_or_else(|_x| -> Duration { Duration::new(0, 1000) });

        self.delta = duration.as_secs_f32();
    }

    pub fn get_delta(&self) -> f32 {
        self.delta
    }
}

impl Default for FpsManager {
    fn default() -> Self {
        FpsManager {
            last_frame: SystemTime::now(),
            delta: 1.0,
        }
    }
}
