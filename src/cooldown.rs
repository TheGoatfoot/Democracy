use std::time::{Duration, SystemTime};

pub struct Cooldown {
    duration: Duration,
    cooldown: SystemTime,
}

impl Cooldown {
    pub fn new(duration: Duration) -> Cooldown {
        Cooldown {
            duration: duration,
            cooldown: SystemTime::UNIX_EPOCH,
        }
    }
    pub fn is_in_cooldown(&self) -> bool {
        self.get_remaining_time() > 0f32
    }
    pub fn get_remaining_time(&self) -> f32 {
        match self.duration.checked_sub(SystemTime::now().duration_since(self.cooldown).unwrap()) {
            Some(duration) => duration.as_secs_f32(),
            None => 0f32
        }
    }
    pub fn put_in_cooldown(&mut self) {
        self.cooldown = SystemTime::now();
    }
    pub fn clear_cooldown(&mut self) {
        self.cooldown = SystemTime::UNIX_EPOCH;
    }
}
