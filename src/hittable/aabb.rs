use glam::Vec3;
use crate::hittable::{HitRecord, Hittable};
use crate::ray::{rand_unit_vec, Ray};
use crate::interval::Interval;
use crate::util::{min, max};

pub struct Aabb {
    x: Interval,
    y: Interval,
    z: Interval,
}



impl Aabb {
    pub fn new(p1: Vec3, p2: Vec3) -> Self {
        Self {
            x: if p1.x < p2.x {Interval{min: p1.x, max: p2.x}} else {Interval{min: p2.x, max: p1.x}},
            y: if p1.y < p2.y {Interval{min: p1.y, max: p2.y}} else {Interval{min: p2.y, max: p1.y}},
            z: if p1.z < p2.z {Interval{min: p1.z, max: p2.z}} else {Interval{min: p2.z, max: p1.z}},
        }
    }

    pub fn unit(a: &Aabb, b: &Aabb) -> Self {
        Self {
            x: Interval { min: min(&a.x.min, &b.x.min), max: max(&a.x.max, &b.x.max) },
            y: Interval { min: min(&a.y.min, &b.y.min), max: max(&a.y.max, &b.y.max) },
            z: Interval { min: min(&a.z.min, &b.z.min), max: max(&a.z.max, &b.z.max) },
        }
    }


    pub fn axis_interval(&self, n: u32) -> &Interval{
        if n == 0 { return &self.x }
        if n == 1 { return &self.y }
        &self.z
    }
}

impl Hittable for Aabb {
    fn hit(&self, ray: &Ray, ray_t: &mut Interval) -> bool {
        let ray_orig = ray.origin;
        let ray_dir = ray.direction;

        for axis in 0..3 {
            let interval_axis = self.axis_interval(axis);
            let mut t0 = (ray_orig.x - interval_axis.min) * ray_dir[axis as usize].recip();
            let mut t1 = (ray_orig.x - interval_axis.max) * ray_dir[axis as usize].recip();

            if t0>t1 { (t1, t0) = (t0, t1); }

            if t0 > ray_t.min { ray_t.min = t0 }
            if t1 < ray_t.max { ray_t.max = t1; }

            if ray_t.size() < 0. { return false; }
        }

        true
        
    }
}
