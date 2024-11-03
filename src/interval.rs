
#[derive(Clone)]
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

    pub fn new(min: f32, max: f32) -> Self {
        Self {min, max}
    }

    pub fn size(&self) -> f32 {
        self.max - self.min
    }

    pub fn intersect(&self, interval: &Interval) -> Interval {
        Interval{
            min: if self.min>interval.min {self.min} else {interval.min},
            max: if self.max<interval.max {self.max} else {interval.max} }
    }

    pub fn contains(&self, x: &f32) -> bool {
        self.min <= *x && *x <= self.max
    }

    pub fn surrounds(&self, x: f32) -> bool {
        self.min < x && x < self.max
    }
}