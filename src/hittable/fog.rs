use crate::hittable::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable, HittableSurfaces};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::util::{rand, rand_unit_vec};
use glam::Vec3;

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
        if let Some(mut rec1) = self.boundaries.hit(ray, &mut Interval::universe()) {
            if let Some(rec2) = self.boundaries.hit(
                ray,
                &mut Interval {
                    min: rec1.t + 0.0001,
                    max: f32::INFINITY,
                },
            ) {
                let hitable_t: Interval = Interval::new(rec1.t, rec2.t).intersect(interval);
                if hitable_t.size() < 0. {
                    return None;
                }

                let ray_length = ray.direction.length();
                let dist_in_boundary = hitable_t.size() * ray_length;
                let hit_dist = self.neg_inv_density * rand().log(std::f32::consts::E);

                //let test = rand();
                //println!("HITTED AT {hit_dist}, BOUNDARIES LEN = {dist_in_boundary}, rand = {}, loge(test) = {}", test, test.log(std::f32::consts::E));

                if hit_dist > dist_in_boundary {
                    return None;
                };
                if rec1.t < 0. {
                    rec1.t = 0.
                }

                let t = rec1.t + hit_dist / ray_length;
                let point = ray.at(t);

                return Some(HitRecord {
                    point,
                    norm: Default::default(),
                    scattered: Ray::new(point, rand_unit_vec()),
                    attenuation: rec1.attenuation,
                    t,
                });
            };
        };

        None
    }

    fn get_bbox(&self) -> Option<Aabb> {
        self.boundaries.get_bbox()
    }
}
