#[derive(Clone, Copy, Default)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub fn universe() -> Self {
        Self {
            min: f32::NEG_INFINITY,
            max: f32::INFINITY,
        }
    }
    
    pub fn ray() -> Self {
        Self {
            min: 0.,
            max: f32::INFINITY,
        }
    }

    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn intersect(&self, interval: &Interval) -> Interval {
        Interval {
            min: self.min.max(interval.min),
            max: self.max.min(interval.max),
        }
    }

    pub fn contains(&self, x: &f32) -> bool {
        self.min <= *x && *x <= self.max
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }
}
