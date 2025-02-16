use std::time::Instant;

const MAX_BUFFER_SIZE: usize = 10;
const INV_MAX_BUFFER_SIZE: f32 = 1f32 / (MAX_BUFFER_SIZE as f32);

pub struct FPSController {
    last_time: Instant,
    buffer: [f32; MAX_BUFFER_SIZE],
    index: usize,
    sum: f32,
}

impl FPSController {
    pub fn default() -> Self {
        Self {
            buffer: [0f32; MAX_BUFFER_SIZE],
            index: 0,
            sum: 0f32,
            last_time: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        let last_fps = self.last_time.elapsed().as_secs_f32().recip();

        self.sum -= self.buffer[self.index];
        self.sum += last_fps;

        self.buffer[self.index] = last_fps;
        self.index += 1usize;
        if self.index == MAX_BUFFER_SIZE {
            self.index = 0usize;
        };
        self.last_time = Instant::now();
    }

    pub fn get_current_fps(&self) -> f32 {
        self.sum * INV_MAX_BUFFER_SIZE
    }
}
