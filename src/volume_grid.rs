use crate::aabb::Aabb;
use bytemuck::{Pod, Zeroable};
use vdb_rs::Grid;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VolumeGridStatic {
    size: [u32; 4],
    shift: [i32; 4],
    bbox: Aabb,
}

pub struct PackedBoolArray {
    pub data: Vec<u32>,
}

impl PackedBoolArray {
    fn from_array(array: &[half::f16], max: f32) -> Self {
        let size = array.len() / 8usize;
        let mut data = Vec::with_capacity(size);

        let mut current_block = 0u32;
        let mut current_block_pos = 0u32;

        for &num in array {
            if current_block_pos == 4u32 {
                data.push(current_block);
                current_block = 0u32;
                current_block_pos = 0u32;
            }
            let shifted_number = (Self::normalize(num, max) as u32) << (24u32 - current_block_pos*8u32);
            current_block |= shifted_number;
            current_block_pos += 1;
        }
        if current_block_pos > 0u32 {
            data.push(current_block);
        }

        Self { data }
    }

    fn normalize(float_num: half::f16, max: f32) -> u8 {
        let native_float = float_num.to_f32();
        let normalized_float = native_float.clamp(0f32, max);
        (normalized_float*255f32/max).round() as u8
    }
}

impl VolumeGridStatic {
    pub fn build_from_vdb_grid(vdb_grid: Grid<half::f16>) -> (Self, Vec<u32>) {
        let min_i = vdb_grid.descriptor.aabb_min().unwrap();
        let max_i = vdb_grid.descriptor.aabb_max().unwrap();

        let length = max_i - min_i;
        let size = [
            length[0] as u32 + 1u32,
            length[1] as u32 + 1u32,
            length[2] as u32 + 1u32,
        ];
        let bbox = Aabb::new(
            [0f32, 0f32, 0f32, 0f32],
            [length.x as f32, length.y as f32, length.z as f32, 0f32],
        );

        let shift = [-min_i.x as i32, -min_i.y as i32, -min_i.z as i32];

        let mut weights =
            vec![
                vec![vec![half::f16::default(); length.z as usize + 1usize]; length.y as usize + 1usize];
                length.x as usize + 1usize
            ];
        let mut max_weight = 0f32;
 
        for (pos, voxel, _level) in vdb_grid.iter() {
            max_weight = f32::max(max_weight, voxel.to_f32());
            weights[(pos.x.floor() + shift[0] as f32) as usize]
                [(pos.y.floor() + shift[1] as f32) as usize]
                [(pos.z.floor() + shift[2] as f32) as usize] = voxel;
        }


        let flattened_weights: Vec<half::f16> = weights.into_iter().flatten().flatten().collect();
        let packed_array = PackedBoolArray::from_array(flattened_weights.as_slice(), max_weight);

        (
            Self {
                size: [size[0], size[1], size[2], 0u32],
                bbox,
                shift: [shift[0], shift[1], shift[2], 0i32],
            },
            packed_array.data,
        )
    }
}
