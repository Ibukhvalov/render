use glam::Vec3;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::{rand_unit_vec, Ray};

#[derive(Clone, Copy)]
pub struct MatteSphere {
    origin: Vec3,
    radius: f32,
    attenuation: Vec3,
}

impl MatteSphere {
    pub fn new(origin: Vec3, radius: f32, attenuation: Vec3) -> Self {
        Self {origin, radius, attenuation}
    }
}
impl Hittable for MatteSphere {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
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
