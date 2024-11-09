use crate::hittable::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable, HittableSurfaces};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util::{rand, rand_unit_vec};

pub struct Fog {
    boundaries: Box<HittableSurfaces>,
    density: f32,
    neg_inv_density: f32,
}

impl Fog {
    pub fn new(boundaries: HittableSurfaces, density: f32) -> Self {
        Self {
            boundaries: Box::new(boundaries),
            density,
            neg_inv_density: -density.recip(),
        }
    }
}
impl Hittable for Fog {
    fn hit(&self, ray: &Ray, interval: &mut Interval) -> Option<HitRecord> {
        if let Some(mut rec) = self.boundaries.hit(ray, &mut Interval::ray()) {

                let ray_length = ray.direction.length();
                let dist_in_boundary = rec.t.size() * ray_length;
                let hit_dist = self.neg_inv_density * rand().log(std::f32::consts::E);

                //let test = rand();
                //println!("HITTED AT {hit_dist}, BOUNDARIES LEN = {dist_in_boundary}, rand = {}, loge(test) = {}", test, test.log(std::f32::consts::E));

                if hit_dist > dist_in_boundary {
                    return None;
                };

                let t = rec.t.min + hit_dist / ray_length;
                let point = ray.at(t);

                return Some(HitRecord {
                    scattered: Ray::new(point, rand_unit_vec()),
                    ..rec
                });
        };

        None
    }

    fn get_bbox(&self) -> Option<Aabb> {
        self.boundaries.get_bbox()
    }
}
