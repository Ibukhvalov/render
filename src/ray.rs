use glam::Vec3;
use crate::hittable::{Hittable, HittableSurfaces};
use crate::interval::Interval;

#[derive(Default, Clone, Copy)]
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

    pub fn get_color(&self, depth: u32, world: &Vec<HittableSurfaces>) -> Vec3 {
        if depth<=0 {
            return Vec3::ZERO
        }

        let mut interval = Interval { min: 0.001, max: f32::INFINITY };

        if let Some(rec) = world.hit(&self, &mut interval) {
            return rec.scattered.get_color(depth-1, world) * rec.attenuation
        }

        let bottom = Vec3::splat(0.8);
        let top = Vec3::new(0.6,0.8,1.);
        let t = (self.direction.y + 1.) / 2.;

        top * t + bottom * (1.-t)
    }
}

