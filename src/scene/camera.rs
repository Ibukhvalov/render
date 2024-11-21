use super::hittable::grid::VolumeGrid;
use crate::interval::Interval;
use super::ray::Ray;
use crate::util::rand_in_square;
use glam::Vec3;
use indicatif::ProgressBar;
use rayon::prelude::*;
use crate::scene::hittable::aabb::Aabb;

pub struct Camera {
    look_from: Vec3,
    look_at: Vec3,
    vup: Vec3,
    vfov: f32,
    aspect: f32,
    top_left_corner: Vec3,
    vertical: Vec3,
    horizontal: Vec3,
    w: Vec3,
    v: Vec3,
    u: Vec3,
    half_height: f32,
    half_width: f32,
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
            vfov,
            top_left_corner,
            vertical,
            horizontal,
            aspect,
            w,
            v,
            u,
            half_width,
            half_height,
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
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.look_from,
            self.top_left_corner + u * self.horizontal + v * self.vertical - self.look_from,
        )
    }
    
    pub fn focus_on(&mut self, bbox: Option<Aabb>) {
        if let Some(bbox) = bbox {
            let (min, max) = (bbox.min, bbox.max);
            
            self.look_at = (max+min) / 2f32;            
            self.look_from = self.look_at.clone() + 400f32*Vec3::Z;
            

            let w = (self.look_from - self.look_at).normalize();
            let u = self.vup.cross(w).normalize();
            let v = u.cross(w);
            

            self.top_left_corner = self.look_from - w - self.half_height * v - self.half_width * u;
            self.vertical = v * self.half_height * 2.;
            self.horizontal = u * self.half_width * 2.;
        }
    }
    
    pub fn update_dist(&mut self, dist: f32) {
        self.top_left_corner = self.look_from - self.w - self.half_height * self.v - self.half_width * self.u;
        self.look_from = self.look_at.clone() + Vec3::Z * dist;
    }

}
