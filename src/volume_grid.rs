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

pub struct PackedBoolArray { 
    pub data: Vec<u32>
 }
 
 impl PackedBoolArray {
    fn from_bool_array(bool_array: &[bool]) -> Self {
       let size = bool_array.len() / 32usize;
       let mut data = Vec::with_capacity(size);
 
       let mut current_pack = 0u32;
       let mut current_bit_index = 0u32;
 
       for &b in bool_array {
          if current_bit_index == 32u32 {
             data.push(current_pack);
             current_bit_index = 0u32;
             current_pack = 0u32;
          }
          let current_bit = (b as u32) << (31u32 - current_bit_index );
          current_pack |= current_bit;
          current_bit_index += 1;
       }
       
       data.push(current_pack);
       
 
       Self { data }
    }

    fn as_slice(&self) -> &[u32] {
        self.data.as_slice()
    }
 }


impl VolumeGridStatic {
    pub fn build_from_vdb_grid(vdb_grid: Grid<half::f16>) -> (Self, Vec<u32>) {
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
                vec![vec![false; length.z as usize + 1usize]; length.y as usize + 1usize];
                length.x as usize + 1usize
            ];

            
        for (pos, _voxel, _level) in vdb_grid.iter() {
            weights[(pos.x.floor() + shift[0] as f32) as usize][(pos.y.floor() + shift[1] as f32) as usize]
                [(pos.z.floor() + shift[2] as f32) as usize] = true;
        }

        let flattened_weights: Vec<bool> = weights.into_iter().flatten().flatten().collect();
        let packed_array = PackedBoolArray::from_bool_array(flattened_weights.as_slice());




        ( Self {
            size: [size[0], size[1], size[2], 0u32],
            bbox,
            shift: [shift[0], shift[1], shift[2], 0i32],
        }, packed_array.data)
    }
}