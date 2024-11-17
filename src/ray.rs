use crate::hittable::grid::VolumeGrid;
use glam::Vec3;

#[derive(Default, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn get_color(&self, grid: &VolumeGrid, background_color: &Vec3) -> Vec3 {
        if let Some(rec) = grid.get_color(self, 2) {
            return background_color * rec.transparency + rec.resulted_color;
        }

        *background_color
    }
}
