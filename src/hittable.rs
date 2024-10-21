use std::ops::Range;
use glam::DVec3;
use crate::ray::{rand_unit_vec, Ray};


pub struct HitRecord {
    pub point: DVec3,
    pub norm: DVec3,
    pub scattered: Ray,
    pub attenuation: DVec3,
    pub t: f64,
}

impl HitRecord {
    pub fn new(point: DVec3, norm: DVec3, scattered: Ray, attenuation: DVec3, t: f64) -> Self{
        Self {point, norm, scattered, attenuation, t}
    }
}

pub trait Hittable {
    fn hit(
        &self,
        ray: &Ray,
        interval: &mut Range<f64>,
    ) -> Option<HitRecord>;
}

pub struct MatteSphere {
    origin: DVec3,
    radius: f64,
    attenuation: DVec3,
}

impl MatteSphere {
    pub fn new(origin: DVec3, radius: f64, attenuation: DVec3) -> Self {
        Self {origin, radius, attenuation}
    }
}

impl Hittable for MatteSphere {
    fn hit(&self, ray: &Ray, interval: &mut Range<f64>) -> Option<HitRecord> {
        let oc = ray.origin - self.origin;
        let a = ray.direction.length_squared();
        let b = 2. * oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius*self.radius;
        let disc = b*b - 4.*a*c;


        let mut root = ( -b - disc.sqrt() ) / (2. * a);
        if !interval.contains(&root) {
            root = ( -b + disc.sqrt() ) / (2. * a);
            if !interval.contains(&root) {
                return None
            }
        }

        let point = ray.at(root);
        let norm = (point - self.origin) / self.radius;
        let scattered = Ray::new(point, point + norm + rand_unit_vec());

        Some(HitRecord::new(point, norm, scattered, self.attenuation, root))
    }
}

impl Hittable for Vec<MatteSphere> {
    fn hit(&self, ray: &Ray, interval: &mut Range<f64>) -> Option<HitRecord> {
        let mut rec = None;

        for sphere in self {
            if let Some(cur_rec) = sphere.hit(ray, interval) {
                interval.end = cur_rec.t;
                rec = Some(cur_rec);
            }
        }

        rec
    }
}


