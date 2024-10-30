

use glam::Vec3;
use crate::ray::{Ray};
use crate::interval::Interval;
pub mod matte_sphere;
pub mod fog;
pub mod aabb;
mod sun;

use matte_sphere::MatteSphere;
use crate::hittable::fog::Fog;

#[derive(Default, Clone, Copy)]
pub struct HitRecord {
    pub point: Vec3,
    pub norm: Vec3,
    pub scattered: Ray,
    pub attenuation: Vec3,
    pub t: f32,
}



impl HitRecord {
    pub fn new(point: Vec3, norm: Vec3, scattered: Ray, attenuation: Vec3, t: f32) -> Self{
        Self {point, norm, scattered, attenuation, t}
    }
}

pub trait Hittable {
    fn hit(
        &self,
        ray: &Ray,
        interval: &mut Interval,
    ) -> Option<HitRecord>;
    fn get_bbox(&self) -> &aabb::Aabb;
}


pub enum HittableSurfaces {
    MatteSphere(MatteSphere),
    Fog(Fog),
    Sun(Sun),
}



impl Hittable for HittableSurfaces {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => {sphere.hit(ray, interval)}
            HittableSurfaces::Fog(fog) => {fog.hit(ray, interval)}
        }
    }

    fn get_bbox(&self) -> &aabb::Aabb {
        match self {
            HittableSurfaces::MatteSphere(sphere) => {&sphere.get_bbox()}
            HittableSurfaces::Fog(fog) => {&fog.get_bbox()}
        }
    }
}


impl Hittable for Vec<HittableSurfaces> {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        let mut rec = None;
        let mut hitted = false;
        for surface in self {
            if let Some(cur_rec) = surface.hit(ray, interval) {
                interval.max = cur_rec.t;
                rec = Some(cur_rec);
                hitted = true;
            }
        }

        rec
    }

    fn get_bbox(&self) -> &aabb::Aabb {
        todo!()
    }
}







