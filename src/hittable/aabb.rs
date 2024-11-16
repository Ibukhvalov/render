use crate::interval::Interval;
use crate::ray::Ray;
use glam::Vec3;

#[derive(Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn unit(a: &Aabb, b: &Aabb) -> Self {
        Self {
            min: Vec3::new(
                f32::min(a.min.x, b.min.x),
                f32::min(a.min.y, b.min.y),
                f32::min(a.min.z, b.min.z),
            ),
            max: Vec3::new(
                f32::max(a.max.x, b.max.x),
                f32::max(a.max.y, b.max.y),
                f32::max(a.max.z, b.max.z),
            ),
        }
    }

    pub fn hit(&self, ray: &Ray, ray_t: &Interval) -> Option<Interval> {
        let ray_orig = ray.origin;
        let ray_dir = ray.direction;

        let mut interval = ray_t.clone();

        for axis in 0..3 {
            let inv_d = ray_dir[axis].recip();
            let (t0, t1) = if inv_d > 0. {
                (
                    (self.min[axis] - ray_orig[axis]) * inv_d,
                    (self.max[axis] - ray_orig[axis]) * inv_d,
                )
            } else {
                (
                    (self.max[axis] - ray_orig[axis]) * inv_d,
                    (self.min[axis] - ray_orig[axis]) * inv_d,
                )
            };

            if t0 > interval.min {
                interval.min = t0;
            }
            if t1 < interval.max {
                interval.max = t1;
            }

            if interval.size() <= 0. {
                return None;
            }
        }

        Some(interval)
    }
}
