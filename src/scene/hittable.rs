use aabb::Aabb;
use glam::Vec3;
pub mod aabb;
pub(crate) mod grid;

#[derive(Clone, Copy)]
pub struct HitRecord {
    pub transparency: f32,
    pub resulted_color: Vec3,
}
