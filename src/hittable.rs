use crate::interval::Interval;
use crate::ray::Ray;
use aabb::Aabb;
use glam::Vec3;
pub mod aabb;
pub mod bvh;
pub mod fog;
pub mod matte_sphere;
mod grid;
mod light_sphere;

use crate::hittable::bvh::BVH;
use crate::hittable::fog::Fog;
use matte_sphere::MatteSphere;
use crate::hittable::grid::Grid;

#[derive(Clone, Copy)]
enum HitRecord {
    Scattered(ScatteredRecord),
    Attenuated(AttenuatedRecord),
}


#[derive(Clone, Copy)]
pub struct ScatteredRecord {
    pub scattered: Ray,
    pub attenuation: Vec3,
    pub t: Interval,
}

pub struct AttenuatedRecord {
    pub emitted: Ray,
    
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord>;
    fn get_bbox(&self) -> Option<Aabb>;
}

pub enum HittableSurfaces {
    MatteSphere(MatteSphere),
    Fog(Fog),
    BVH(BVH),
    Grid(Grid),
}

impl Hittable for HittableSurfaces {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => sphere.hit(ray, interval),
            HittableSurfaces::Fog(fog) => fog.hit(ray, interval),
            HittableSurfaces::BVH(bvh) => bvh.hit(ray, interval),
        }
    }

    fn get_bbox(&self) -> Option<Aabb> {
        match self {
            HittableSurfaces::MatteSphere(sphere) => sphere.get_bbox(),
            HittableSurfaces::Fog(fog) => fog.get_bbox(),
            HittableSurfaces::BVH(bvh) => bvh.get_bbox(),
        }
    }
}

impl Hittable for Vec<HittableSurfaces> {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        let mut rec = None;
        for surface in self.iter() {
            if let Some(cur_rec) = surface.hit(ray, interval) {
                interval.max = cur_rec.t.max;
                rec = Some(cur_rec);
            }
        }
        rec
    }

    fn get_bbox(&self) -> Option<Aabb> {
        panic!("WHERE?")
    }
}
