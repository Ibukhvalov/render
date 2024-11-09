use crate::hittable::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util::rand_unit_vec;
use glam::Vec3;

pub struct MatteSphere {
    origin: Vec3,
    radius: f32,
    attenuation: Vec3,
}

impl MatteSphere {
    pub fn new(origin: Vec3, radius: f32, attenuation: Vec3) -> Self {
        Self {
            origin,
            radius,
            attenuation,
        }
    }
}
impl Hittable for MatteSphere {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        let oc = ray.origin - self.origin;
        let a = ray.direction.length_squared();
        let b = 2. * oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let disc = b * b - 4. * a * c;

        let root1 = (-b - disc.sqrt()) / (2. * a);
        let root2 = (-b + disc.sqrt()) / (2. * a);
        
        let hit_interval = interval.intersect(&Interval::new(root1, root2));
        if(disc < 0. || hit_interval.size() < 0.) {
            return None;
        }

        let point = ray.at(hit_interval.min);
        let norm = (point - self.origin) / self.radius;
        let scattered = Ray::new(point, point + norm + rand_unit_vec());

        Some(HitRecord {
            scattered,
            attenuation: self.attenuation,
            t: hit_interval,
        })
    }

    fn get_bbox(&self) -> Option<Aabb> {
        Some(Aabb::new(
            self.origin - self.radius,
            self.origin + self.radius,
        ))
    }
}
