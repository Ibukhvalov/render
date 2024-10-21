use std::ops;
use glam::{DVec2, DVec3};
use rand::Rng;
use crate::hittable::{Hittable, MatteSphere};
use ops::Range;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        Self { origin, direction: direction.normalize() }
    }

    pub fn at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    pub fn get_color(&self, depth: u32, world: &Vec<MatteSphere>) -> DVec3 {
        if depth<=0 {
            return DVec3::ZERO
        }

        let mut interval = Range { start: 0.00001, end: f64::INFINITY };

        if let Some(rec) = world.hit(&self, &mut interval) {
            return rec.scattered.get_color(depth-1, world) * rec.attenuation
        }

        let bottom = DVec3::splat(0.8);
        let top = DVec3::new(0.8,0.9,1.);
        let t = (self.direction.y + 1.) / 2.;

        top * t + bottom * (1.-t)
    }
}


pub fn rand_unit_vec() -> DVec3 {
    let mut rng = rand::thread_rng();
    loop {
        let vec = DVec3::new(
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
        );

        if vec.length_squared() < 1. {
            break vec;
        }
    }
}

pub fn rand_in_square() -> DVec2 {
    let mut rng = rand::thread_rng();
    DVec2::new(rng.gen(), rng.gen())
}