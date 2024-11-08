use crate::camera::Camera;
use crate::hittable::HittableSurfaces;
use glam::Vec3;
use hittable::matte_sphere::MatteSphere;
use std::fs::File;
use std::io::BufReader;
use vdb_rs::VdbReader;

mod camera;
mod hittable;
mod interval;
mod ray;
mod util;

use crate::hittable::bvh::BVH;
use crate::hittable::fog::Fog;

fn main() {
    const ASPECT: f32 = 14. / 9.;
    const IMG_WIDTH: u32 = 400;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT) as u32;

    // read from vdb file
    let filename = std::env::args()
        .nth(1)
        .expect("Missing VDB filename as first argument");

    let f = File::open(filename).unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();
    let grid_names = vdb_reader.available_grids();

    let grid_to_load = grid_names.first().cloned().unwrap_or(String::new());

    let grid = vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap();

    let mut world: Vec<HittableSurfaces> = grid
        .iter()
        .map(|(pos, _voxel, level)| {
            let pos_vec3 = Vec3::new(pos.x, pos.y, pos.z);
            HittableSurfaces::MatteSphere(MatteSphere::new(
                (pos_vec3 + level.scale()) * 0.1,
                level.scale() * 0.1,
                Vec3::splat(0.4),
            ))
        })
        .collect();

    /*

    let mut world = vec![
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(0.,-1000.,0.), 1000., Vec3::new(0.5,0.5,0.5))),
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(0.,1.,0.), 1., Vec3::new(0.9,0.1,0.9))),
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(2.,1.,0.), 1., Vec3::new(0.1,0.5,0.9))),
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(-2.,1.,0.), 1., Vec3::new(0.6,0.1,0.6))),
        HittableSurfaces::MatteSphere(MatteSphere::new(Vec3::new(0.,3.,0.), 1., Vec3::new(0.2,0.5,0.1))),

    ];


    */
    eprintln!("[1/2]🌳 Building bvh with {} objects", world.len());
    world = vec![HittableSurfaces::BVH(BVH::init_from_vec(world, 0))];

    let camera = Camera::new(
        Vec3::new(-5., 15., 70.),
        Vec3::new(-5., 22.5, 0.),
        Vec3::Y,
        60.,
        ASPECT,
    );

    camera.render_to_out(world, IMG_WIDTH, IMG_HEIGHT, 200);
}
