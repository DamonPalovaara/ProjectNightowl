#[cfg(target_arch = "wasm32")]
use instant::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

pub struct Time {
    init: Instant,
    last_tick: Instant,
}

impl Time {
    pub fn new() -> Self {
        let init = Instant::now();
        let last_tick = init;
        Self { init, last_tick }
    }

    pub fn tick(&mut self) -> f32 {
        let delta_time = self.last_tick.elapsed().as_secs_f32();
        self.last_tick = Instant::now();
        delta_time
    }

    pub fn run_time(&self) -> f32 {
        self.init.elapsed().as_secs_f32()
    }
}
