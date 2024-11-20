use super::hittable::grid::VolumeGrid;
use crate::interval::Interval;
use super::ray::Ray;
use crate::util::rand_in_square;
use glam::Vec3;
use indicatif::ProgressBar;
use rayon::prelude::*;

pub struct Camera {
    look_from: Vec3,
    look_at: Vec3,
    vup: Vec3,

    top_left_corner: Vec3,

    vertical: Vec3,
    horizontal: Vec3,
}

impl Camera {
    pub fn new(look_from: Vec3, look_at: Vec3, vup: Vec3, vfov: f32, aspect: f32) -> Self {
        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = u.cross(w);

        let half_height = (vfov * std::f32::consts::PI / 180. / 2.).tan();
        let half_width = half_height * aspect;

        let top_left_corner = look_from - w - half_height * v - half_width * u;
        let vertical = v * half_height * 2.;
        let horizontal = u * half_width * 2.;

        Self {
            look_from,
            look_at,
            vup,

            top_left_corner,
            vertical,
            horizontal,
        }
    }

    pub fn update(&mut self, vfov: f32, aspect: f32) {
        let w = (self.look_from - self.look_at).normalize();
        let u = self.vup.cross(w).normalize();
        let v = u.cross(w);

        let half_height = (vfov * std::f32::consts::PI / 180. / 2.).tan();
        let half_width = half_height * aspect;

        self.top_left_corner = self.look_from - w - half_height * v - half_width * u;
        self.vertical = v * half_height * 2.;
        self.horizontal = u * half_width * 2.;
    }

    // #->(u)#
    // I######
    // v######
    // (v)####
    pub(crate) fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.look_from,
            self.top_left_corner + u * self.horizontal + v * self.vertical - self.look_from,
        )
    }

}
