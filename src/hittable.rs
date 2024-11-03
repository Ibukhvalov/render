

use glam::Vec3;
use crate::ray::{Ray};
use crate::interval::Interval;
use aabb::Aabb;
pub mod matte_sphere;
pub mod fog;
pub mod aabb;
pub(crate) mod bvh;

use matte_sphere::MatteSphere;
use crate::hittable::bvh::BVH;
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
    fn get_bbox(&self) -> Option<Aabb>;
}


pub enum HittableSurfaces {
    MatteSphere(MatteSphere),
    Fog(Fog),
    BVH(BVH),
}



impl Hittable for HittableSurfaces {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => {sphere.hit(ray, interval)}
            HittableSurfaces::Fog(fog) => {fog.hit(ray, interval)}
            HittableSurfaces::BVH(bvh) => {bvh.hit(ray, interval)}
        }
    }

    fn get_bbox(&self) -> Option<Aabb> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => {sphere.get_bbox()}
            HittableSurfaces::Fog(fog) => {fog.get_bbox()}
            HittableSurfaces::BVH(bvh) => {bvh.get_bbox()}
        }
    }
}


impl Hittable for Vec<HittableSurfaces> {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        let mut rec = None;
        for surface in self.iter() {
            if let Some(cur_rec) = surface.hit(ray, interval) {
                interval.max = cur_rec.t;
                rec = Some(cur_rec);
            }
        }
        rec
    }

    fn get_bbox(&self) -> Option<Aabb> {
        panic!("WHERE?")
    }
}




