

use glam::Vec3;
use crate::ray::{Ray};
use crate::interval::Interval;
pub mod matte_sphere;
pub mod aabb;
pub(crate) mod bvh_node;

use matte_sphere::MatteSphere;

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
}

#[derive(Copy, Clone)]
pub enum HittableSurfaces {
    MatteSphere(MatteSphere),
    BVHNode(bvh_node),
}



impl Hittable for HittableSurfaces {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => {sphere.hit(ray, interval)}
            HittableSurfaces::MatteCuboid(cub) => {cub.hit(ray, interval)}
        }
    }
}


impl Hittable for Vec<HittableSurfaces> {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        let mut rec = None;

        for surface in self {
            if let Some(cur_rec) =  surface.hit(ray, interval) {
                interval.max = cur_rec.t;
                rec = Some(cur_rec);
            }
        }
        rec
    }
}






