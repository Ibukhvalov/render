use crate::camera::Camera;
use crate::hittable::grid::VolumeGrid;
use glam::Vec3;
use std::fs::File;
use std::io::BufReader;
use vdb_rs::{Grid, GridDescriptor, VdbReader};

mod camera;
mod hittable;
mod interval;
mod ray;
mod util;


fn get_view_positions_from_to(descriptor: &GridDescriptor, vfov: f32) -> (Vec3,Vec3) {
    let min = descriptor.aabb_min().unwrap();
    let max = descriptor.aabb_max().unwrap();

    let center = Vec3::new((min.x+max.x) as f32/ 2.0, (min.y+max.y) as f32/ 2.0,(min.z+max.z) as f32/ 2.0);

    let height = (max.y - min.y) as f32;
    let dist = (vfov.to_radians()/2f32).tan() * height/2.0;
    
    let mut camera_pos = center + Vec3::Z * (dist + (max.z as f32) - center.z) * 1.2f32;
    camera_pos.y = 0.0;
    
    (camera_pos, center)
}

    fn load_vdb_grid_from_args() -> Grid<half::f16>{
        let filename = std::env::args()
            .nth(1)
            .expect("Missing VDB filename as first argument");
    
        let mut vdb_reader = VdbReader::new(BufReader::new(File::open(filename).unwrap())).unwrap();
        let grid_to_load = vdb_reader.available_grids().first().cloned().unwrap_or(String::new());
        vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap()
    }



fn main() {
    const ASPECT: f32 = 14. / 9.;
    const IMG_WIDTH: u32 = 350;
    const IMG_HEIGHT: u32 = (IMG_WIDTH as f32 / ASPECT) as u32;
    const VFOV: f32 = 80.0;


    let grid_vdb = load_vdb_grid_from_args();
//    let (camera_pos, target_pos) = get_view_positions_from_to(&grid_vdb.descriptor, VFOV);

    let (camera_pos, target_pos) = (Vec3::new(0.0,0.0,100.0), Vec3::ZERO);
    let grid = VolumeGrid::build_from_vdb_grid(grid_vdb, 0.5);

    let camera = Camera::new(camera_pos, target_pos, Vec3::Y, VFOV, ASPECT);

    camera.render_to_out(&grid, IMG_WIDTH, IMG_HEIGHT, 16);
}
