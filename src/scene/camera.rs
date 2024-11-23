use super::ray::Ray;
use crate::scene::hittable::aabb::Aabb;
use glam::{Mat4, Vec3};

pub struct Camera {
    pub camera_to_world: Mat4,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, _vup: Vec3, _vfov: f32, _aspect: f32) -> Self {
        Self {
            camera_to_world: Mat4::look_at_rh(look_from, look_at, Vec3::Y),
        }
    }

    // #->(u)#
    // I######
    // v######
    // (v)####
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.camera_to_world.transform_point3(Vec3::ZERO),
            self.camera_to_world.transform_vector3(Vec3::new(
                u * 2f32 - 1f32,
                -(v * 2f32 - 1f32),
                1f32,
            )),
        )
    }
}
