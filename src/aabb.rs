use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Aabb {
    min: [f32; 4],
    max: [f32; 4],
}

impl Aabb {
    pub fn new(min: [f32; 4], max: [f32; 4]) -> Self {
        Self { min, max }
    }
}
