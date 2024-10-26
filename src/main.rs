use std::fs::File;
use std::io::BufReader;
use glam::Vec3;
use crate::camera::Camera;
use hittable::matte_sphere::MatteSphere;
use crate::hittable::HittableSurfaces;
use crate::hittable::aabb;
use crate::hittable::bvh_node;
use crate::interval::Interval;

mod ray;
mod hittable;
mod camera;
mod interval;
mod util;

use vdb_rs::VdbReader;

fn main() {
    const ASPECT: f32 = 6./9.;
    const IMG_WIDTH: u32 = 250;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT) as u32;



    let filename = std::env::args()
        .nth(1)
        .expect("Missing VDB filename as first argument");

    let f = File::open(filename).unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let grid_names = vdb_reader.available_grids();

    let grid_to_load = grid_names.first().cloned().unwrap_or(String::new());


    let grid = vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap();

    let world: Vec<HittableSurfaces> = grid
        .iter()
        .map(|(pos, voxel, level)| {
            let pos_vec3 = Vec3::new(pos.x, pos.y, pos.z);
            HittableSurfaces::MatteSphere(MatteSphere::new(
                (pos_vec3 + level.scale()) * 0.1,
                level.scale()*0.1,
                Vec3::splat(0.3),
            ))
        })
        .collect();



    let camera = Camera::new(Vec3::new(0.,0.5,16.), Vec3::new(9.5,14.,0.), Vec3::Y, 90., ASPECT);

    camera.render_to_out(&world, IMG_WIDTH, IMG_HEIGHT, 6);



}
