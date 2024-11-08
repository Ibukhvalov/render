use crate::hittable::{Hittable, HittableSurfaces};
use glam::{Vec2, Vec3};
use rand::Rng;
use std::cmp::Ordering;

pub fn rand_unit_vec() -> Vec3 {
    let mut rng = rand::thread_rng();
    loop {
        let vec = Vec3::new(
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
            rng.gen_range(-1.0..1.),
        );

        if vec.length_squared() < 1. {
            break vec;
        }
    }
}

pub fn rand_in_square() -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(rng.gen(), rng.gen())
}

pub fn rand() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn box_compare(axis: usize) -> impl FnMut(&HittableSurfaces, &HittableSurfaces) -> Ordering {
    move |a, b| {
        let abbox = a.get_bbox();
        let bbbox = b.get_bbox();
        if let (Some(a), Some(b)) = (abbox, bbbox) {
            let ac = a.min[axis] + a.max[axis];
            let bc = b.min[axis] + b.max[axis];
            ac.partial_cmp(&bc).unwrap()
        } else {
            panic!("no bbox in node")
        }
    }
}

pub fn axis_range(hittable_list: &Vec<HittableSurfaces>, axis: usize) -> f32 {
    let mut infinum = f32::INFINITY;
    let mut supremum = f32::NEG_INFINITY;

    for surface in hittable_list {
        if let Some(bbox) = surface.get_bbox() {
            if bbox.min[axis] < infinum {
                infinum = bbox.min[axis];
            }
            if bbox.max[axis] > supremum {
                supremum = bbox.min[axis];
            }
        }
    }
    supremum - infinum
}
