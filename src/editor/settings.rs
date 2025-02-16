use glam::{Mat4, Vec3};
use super::FPSController;


pub struct Settings {
    pub background_color: Vec3,
    pub light_color: Vec3,
    pub lightness: f32,
    pub g: f32,
    pub absorption: f32,
    pub scattering: f32,
    pub _spp: u32,
    pub ray_marching_step: f32,
    pub picked_path: Option<String>,
    pub matrix: Mat4,
    pub fps_ctrl: FPSController,
}

impl Settings {
    pub fn default() -> Self {
        Self {
                background_color: Vec3::new(0.7f32, 0.7f32, 0.9f32),
                light_color: Vec3::new(1.0, 0.9, 0.9),
                lightness: 10f32,
                g: 0.6,
                absorption: 0.02,
                scattering: 0.4,
                ray_marching_step: 10f32,
                _spp: 1u32,
                picked_path: None,
                matrix: Mat4::IDENTITY,
                fps_ctrl: FPSController::default(),
        }
    }
}