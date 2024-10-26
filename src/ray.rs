use std::ops;
use glam::{Vec2, Vec3};
use rand::Rng;
use crate::hittable;
use crate::hittable::Hittable;
use crate::interval::Interval;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction: direction.normalize() }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn get_color(&self, depth: u32, world: &Vec<hittable::HittableSurfaces>) -> Vec3 {
        if depth<=0 {
            return Vec3::ZERO
        }

        let mut interval = Interval { min: 0.00001, max: f32::INFINITY };

        if let Some(rec) = world.hit(&self, &mut interval) {
            return rec.scattered.get_color(depth-1, world) * rec.attenuation
            //return rec.attenuation;
        }

        let bottom = Vec3::splat(0.8);
        let top = Vec3::new(0.8,0.9,1.);
        let t = (self.direction.y + 1.) / 2.;

        top * t + bottom * (1.-t)
    }
}


pub fn rand_unit_vec() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let vec = Vec3::new(
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
        );

        if vec.length_squared() < 1. {
            break vec;
        }
    }
}

pub fn rand_in_square() -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(rng.gen(), rng.gen())
}