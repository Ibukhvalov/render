use glam::{Quat, Vec3};

pub struct View {
    pub rotation_x: Quat,
    pub rotation_y: Quat,
    pub translation: Vec3,
}

impl View {
    pub fn default() -> Self {
        Self {
            rotation_x: Quat::IDENTITY,
            rotation_y: Quat::IDENTITY,
            translation: Vec3::NEG_Z * 150f32,
        }
    }
}
