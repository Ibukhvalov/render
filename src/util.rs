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

