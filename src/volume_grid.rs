use bytemuck::{Pod, Zeroable};
use vdb_rs::Grid;
use crate::aabb::Aabb;

const BASE_WEIGHT: f32 = 0.1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VolumeGridStatic {
    size: [u32; 4],
    shift: [i32; 4],
    bbox: Aabb,
    //weights: Vec<Vec<Vec<f32>>>,
}

impl VolumeGridStatic {
    pub fn build_from_vdb_grid(vdb_grid: Grid<half::f16>) -> (Self, Vec<Vec<Vec<f32>>>) {
        let min_i = vdb_grid.descriptor.aabb_min().unwrap();
        let max_i = vdb_grid.descriptor.aabb_max().unwrap();

        let bbox = Aabb::new(
            [min_i.x as f32, min_i.y as f32, min_i.z as f32, 0f32],
            [max_i.x as f32, max_i.y as f32, max_i.z as f32, 0f32],
        );
    
        let length = max_i - min_i;
        let size = [length[0] as u32 + 1u32, length[1] as u32 + 1u32, length[2] as u32 + 1u32];
        let bbox = Aabb::new(
            [0f32,0f32,0f32,0f32],
            [length.x as f32, length.y as f32, length.z as f32, 0f32]
        );

    
        let shift = [-min_i.x as i32, -min_i.y as i32, -min_i.z as i32];

        let mut weights =
            vec![
                vec![vec![0f32; length.z as usize + 1usize]; length.y as usize + 1usize];
                length.x as usize + 1usize
            ];

            
        for (pos, _voxel, _level) in vdb_grid.iter() {
            weights[(pos.x.floor() + shift[0] as f32) as usize][(pos.y.floor() + shift[1] as f32) as usize]
                [(pos.z.floor() + shift[2] as f32) as usize] = BASE_WEIGHT;
        }


        ( Self {
            size: [size[0], size[1], size[2], 0u32],
            bbox,
            shift: [shift[0], shift[1], shift[2], 0i32],
        }, weights )
    }
}