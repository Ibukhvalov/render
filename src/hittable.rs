use crate::interval::Interval;
use crate::ray::Ray;
use aabb::Aabb;
use glam::Vec3;
pub mod aabb;
pub(crate) mod grid;
mod light_sphere;

#[derive(Clone, Copy)]
pub struct HitRecord {
    pub transparency: f32,
    pub resulted_color: Vec3,
}
