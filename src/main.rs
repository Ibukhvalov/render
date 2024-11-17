use crate::camera::Camera;
use crate::hittable::grid::VolumeGrid;
use glam::Vec3;
use std::fs::File;
use std::io::BufReader;
use vdb_rs::VdbReader;

mod camera;
mod hittable;
mod interval;
mod ray;
mod util;

fn main() {
    const ASPECT: f32 = 14. / 9.;
    const IMG_WIDTH: u32 = 10;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT) as u32;

    // read from vdb file
    //let filename = std::env::args()
    //    .nth(1)
    //    .expect("Missing VDB filename as first argument");

    let filename = "./data/vdbAssets/wdas_cloud_sixteenth.vdb";

    let f = File::open(filename).unwrap();
    let mut vdb_reader = VdbReader::new(BufReader::new(f)).unwrap();

    let grid_names = vdb_reader.available_grids();
    let grid_to_load = grid_names.first().cloned().unwrap_or(String::new());

    let grid_vdb = vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap();

    let min = grid_vdb.descriptor.aabb_min().unwrap();
    let max = grid_vdb.descriptor.aabb_max().unwrap();

    let center = (max + min) / 2;
    let center_vec3 = Vec3::new(center.x as f32, center.y as f32, center.z as f32);
    eprintln!("{:?} {:?}", min, max);
    //eprintln!("{center:?}");

    let grid = VolumeGrid::build_from_vdb_grid(grid_vdb, 0.5);

    let camera = Camera::new(Vec3::new(0., 0., 200.), center_vec3, Vec3::Y, 40., ASPECT);

    camera.render_to_out(&grid, IMG_WIDTH, IMG_HEIGHT, 1);
}
