use std::fs::File;
use std::io::BufReader;
use glam::Vec3;
use rayon::iter::IntoParallelIterator;
use vdb_rs::VdbReader;
use crate::scene::camera::Camera;
use crate::scene::hittable::grid::VolumeGrid;
use crate::scene::ray::Ray;

pub(crate) mod camera;
mod hittable;
mod ray;

pub struct Scene {
    grid: VolumeGrid,
    background: Vec3,
}

impl Scene {
    pub fn new(background: Vec3) -> Self {
        let filename = std::env::args()
            .nth(1)
            .expect("Missing VDB filename as first argument");

        let mut vdb_reader = VdbReader::new(BufReader::new(File::open(filename).unwrap())).unwrap();
        let grid_to_load = vdb_reader.available_grids().first().cloned().unwrap_or(String::new());

        let grid = VolumeGrid::build_from_vdb_grid(vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap(), 0.5);

        Self {
            grid,
            background,
        }
    }


    pub fn change_background(&mut self, bground: Vec3) {
        self.background = bground;
    }

    pub fn get_color(&self, ray: Ray) -> Vec3 {
        ray.get_color(&self.grid, &self.background)
    }


}