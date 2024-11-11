use half::f16;
use vdb_rs::Grid;
use crate::hittable::{Aabb, HitRecord};
use glam::{IVec3, Vec3};
use indicatif::ProgressBar;
use crate::interval::Interval;
use crate::ray::Ray;

pub struct VolumeGrid {
    bbox: Aabb,
    weights: Vec<Vec<Vec<Option<f32>>>>,
    shift: Vec3,
    light_dir: Vec3,
    light_col: Vec3,
}


impl VolumeGrid {
    pub fn build_from_vdb_grid(vdb_grid: Grid<f16>, base_weight: f32) -> Self {
        let min_i = vdb_grid.descriptor.aabb_min().unwrap();
        let max_i = vdb_grid.descriptor.aabb_max().unwrap();

        let bbox = Aabb::new(Vec3::new(min_i.x as f32, min_i.y as f32, min_i.z as f32), Vec3::new(max_i.x as f32, max_i.y as f32, max_i.z as f32));
        let length = max_i - min_i;
        let shift = -Vec3::new(min_i.x as f32, min_i.y as f32, min_i.z as f32);

        let mut weights = vec![vec![vec![None; length.z as usize +1usize]; length.y as usize+1usize]; length.x as usize+1usize];
        eprintln!("[1/2] 🏽 Building grid...");

        /*
        let pb = ProgressBar::new(vdb_grid. as u64);

        pb.inc(width as u64);

        pb.finish_with_message("✅ Complete!");

         */
        for el in vdb_grid.iter() {
            let (pos, _voxel, _level) = el;

            weights[(pos.x.floor()+shift.x) as usize][(pos.y.floor()+shift.y) as usize][(pos.z.floor()+shift.z) as usize] = Some(base_weight);
        }

        Self {bbox, weights, shift, light_dir: Vec3::new(0.1,0.2,-1.), light_col: Vec3::splat(0.5)}
    }

    fn get_weight(&self, pos: Vec3) -> Option<f32> {
        let indexes = pos + self.shift;
        self.weights[indexes.x.floor() as usize][indexes.y.floor() as usize][indexes.z.floor() as usize]
    }

    pub fn get_color(&self, ray: &Ray, depth: u32) -> Option<HitRecord> {
        if depth == 0 { return None }

        if let Some(interval_bbox) = self.bbox.hit(ray, &Interval::ray()) {
            let step_size = 1f32;

            let t0 = interval_bbox.min;
            let t1 = interval_bbox.max;

            let mut transparency = 1f32;
            let mut result = Vec3::ZERO;
            let ns = ((t1 - t0)/step_size).round() as u32;

            for n in 0..ns {
                let t = t1.min(t0 + step_size * (n as f32 + 0.5));
                let sample_pos = ray.at(t);
                if let Some(sample_weight) = self.get_weight(sample_pos) {
                    let sample_transparency = (-step_size * sample_weight).exp();
                    transparency *= sample_transparency;

                    if let Some(rec) = self.get_color(&Ray::new(sample_pos, self.light_dir), depth-1) {
                        let light_attenuation = rec.transparency;
                        result += transparency * self.light_col * light_attenuation * step_size;
                    }

                }
            }
            return Some(HitRecord {
                transparency,
                resulted_color: result,
            })
        }

        None
    }

    fn get_bbox(&self) -> Option<Aabb> {
        Some(self.bbox.clone())
    }
}