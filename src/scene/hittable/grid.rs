use crate::interval::Interval;
use crate::scene::hittable::{Aabb, HitRecord};
use crate::scene::ray::Ray;
use crate::util::cos_between;
use glam::Vec3;
use half::f16;
use std::f32::consts::PI;
use vdb_rs::Grid;

type Weight = f32;

pub struct VolumeGrid {
    bbox: Aabb,
    weights: Vec<Vec<Vec<Option<Weight>>>>,
    resolution: [f32; 3],
    shift: Vec3,
    pub light_dir: Vec3,
    pub light_col: Vec3,
    pub absorption: f32,
    pub scattering: f32,
    pub g: f32,
    pub step_size: f32,
}

impl VolumeGrid {
    pub fn build_from_vdb_grid(vdb_grid: Grid<f16>, base_weight: f32) -> Self {
        eprintln!("[1/3] 🏽 Building grid...");

        let min_i = vdb_grid.descriptor.aabb_min().unwrap();
        let max_i = vdb_grid.descriptor.aabb_max().unwrap();

        let bbox = Aabb::new(
            Vec3::new(min_i.x as f32, min_i.y as f32, min_i.z as f32),
            Vec3::new(max_i.x as f32, max_i.y as f32, max_i.z as f32),
        );
        let length = max_i - min_i;
        let resolution = [length[0] as f32, length[1] as f32, length[2] as f32];

        let shift = -Vec3::new(min_i.x as f32, min_i.y as f32, min_i.z as f32);

        let mut weights =
            vec![
                vec![vec![None; length.z as usize + 1usize]; length.y as usize + 1usize];
                length.x as usize + 1usize
            ];

        for el in vdb_grid.iter() {
            let (pos, _voxel, _level) = el;

            weights[(pos.x.floor() + shift.x) as usize][(pos.y.floor() + shift.y) as usize]
                [(pos.z.floor() + shift.z) as usize] = Some(base_weight);
        }

        Self {
            resolution,
            bbox,
            weights,
            shift,
            light_dir: Vec3::ONE,
            light_col: Vec3::new(3.0, 0.0, 0.0),
            absorption: 0.13,
            scattering: 0.8,
            g: 0.6,
            step_size: 10f32,
        }
    }

    // the Henyey-Greenstein phase function
    fn phase(&self, cos_theta: f32) -> f32 {
        let g = self.g;
        let denom = 1f32 + g * g - 2f32 * g * cos_theta;
        1f32 / (4f32 * PI) * (1f32 - g * g) / (denom * denom.sqrt())
    }
    fn get_weight(&self, pos: Vec3) -> Option<f32> {
        let indexes = (pos.round());
        if(indexes.x < 0f32 || indexes.x > self.resolution[0] ||
            indexes.y < 0f32 || indexes.y > self.resolution[1] ||
            indexes.z < 0f32 || indexes.z > self.resolution[2])
        {None}
        else {
            self.weights[indexes.x as usize][indexes.y as usize][indexes.z as usize]
        }
    }
    
    fn get_interpolated_weight(&self, pos: Vec3) -> Option<f32> {
        let p_local = pos - self.bbox.min;
        let p_lattice = p_local - 0.5f32;

        let p_rounded = p_lattice.floor();
        
        let mut weight = [0f32; 3];
        let mut result = 0f32;
        
        for i in 0..2 {
            weight[0] = 1f32 - (p_lattice.x - (p_rounded.x + i as f32)).abs();
            for j in 0..2 {
                weight[1] = 1f32 - (p_lattice.y - (p_rounded.y + j as f32)).abs();
                for k in 0..2 {
                    weight[2] = 1f32 - (p_lattice.z - (p_rounded.z + k as f32)).abs();
                    let prob_weight = self.get_weight(Vec3::new(i as f32,j as f32,k as f32) + p_rounded);
                    if let Some(prob_weight) = prob_weight {
                        result += weight[0] * weight[1] * weight[2] * prob_weight;
                    }
                }
            }
        }
        if result > 0e-3 {
            Some(result)
        } else {
            None
        }
    }

    pub fn get_color(&self, ray: &Ray, depth: u32) -> Option<HitRecord> {
        if depth == 0 {
            return None;
        }

        if let Some(interval_bbox) = self.bbox.hit(ray, &Interval::ray()) {
            let t0 = interval_bbox.min;
            let t1 = interval_bbox.max;

            let mut transparency = 1f32;
            let mut result = Vec3::ZERO;
            let ns = ((t1 - t0) / self.step_size).round() as u32;

            for n in 0..ns {
                if transparency <= 0.001 {
                    break;
                }
                let t = t1.min(t0 + self.step_size * (n as f32 + 0.5));
                let sample_pos = ray.at(t);
                //if (sample_pos).length_squared() <= 100f32 {
                if let Some(sample_density) = self.get_interpolated_weight(sample_pos) {
                    //let sample_density = 0.1f32;
                    let sample_transparency =
                        (-self.step_size * sample_density * (self.scattering * self.absorption))
                            .exp();
                    transparency *= sample_transparency;

                    if let Some(rec) =
                        self.get_color(&Ray::new(sample_pos, self.light_dir), depth - 1)
                    {
                        let light_attenuation = rec.transparency;
                        let cos_theta = cos_between(&ray.direction, &self.light_dir);
                        result += transparency
                            * self.light_col
                            * light_attenuation
                            * self.step_size
                            * sample_density
                            * self.phase(cos_theta)
                            * self.scattering;
                    }
                }
            }
            return Some(HitRecord {
                transparency,
                resulted_color: result,
            });
        }

        None
    }

    pub fn get_bbox(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}
