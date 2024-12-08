use crossbeam_channel::Sender;

use super::scene::camera::Camera;
use super::scene::Scene;
use crate::interval::Interval;
use crate::util::*;

use crate::Settings;
use glam::{Mat4, Vec3};
use log::debug;
use num_traits::real::Real;
use rayon::prelude::*;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    color: [f32; 4],
}
impl Color {
    fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color: [r, g, b, a],
        }
    }
}

pub struct PathTracerRenderContext {
    // scene: Arc<EmbreeScene>,
    image_data: Mutex<Vec<Color>>,
    result_width: u32,
    result_height: u32,
    input_rx: single_value_channel::Receiver<Mat4>,
    tx: Sender<Vec<Color>>,
    settings: Arc<Mutex<Settings>>,
}
impl PathTracerRenderContext {
    pub fn new(
        width: u32,
        height: u32,
        tx: Sender<Vec<Color>>,
        input_rx: single_value_channel::Receiver<Mat4>,
        settings: Arc<Mutex<Settings>>,
    ) -> Self {
        Self {
            result_height: height,
            result_width: width,
            image_data: Mutex::new(vec![Color::default(); (width * height) as usize]),
            tx,
            input_rx,
            settings,
        }
    }
}

pub struct Renderer {
    camera: Camera,
    scene: Scene,
    samples_per_pixel: u32,
}

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new(Vec3::new(0., 0., 1000.), Vec3::ZERO, Vec3::Y, 80., 1.0);
        Self {
            samples_per_pixel: 3,
            camera,
            scene: Scene::new(Vec3::new(0.6, 0.6, 0.9)),
        }
    }

    fn update_settings(&mut self, pt_ctx: &mut PathTracerRenderContext) {
        if let Ok(mut settings) = pt_ctx.settings.lock() {
            if let Some(path) = &settings.picked_path {
                self.scene.update_scene(path);
                settings.picked_path = None;
            }
            self.scene.background = settings.background_color;
            self.scene.grid.light_col = settings.light_color * 1.3f32;
            self.scene.grid.g = settings.g;
            self.scene.grid.absorption = settings.absorption;
            self.scene.grid.scattering = settings.scattering;
            self.scene.grid.step_size = settings.ray_marching_step;
            self.samples_per_pixel = settings.spp.ceil() as u32;
        } else {
            panic!("Could not acquire settings lock, skipping this frame.");
        }
        self.camera.camera_to_world = *pt_ctx.input_rx.latest();
    }
    pub fn run_iteration(&mut self, pt_ctx: &mut PathTracerRenderContext) {
        let mut image_data = pt_ctx.image_data.lock().unwrap().clone();
        self.update_settings(pt_ctx);
        let border = Interval::new(0., 0.9999);
        let (width, height) = (pt_ctx.result_width, pt_ctx.result_height);

        let mut current_progress: f32 = 0.;
        let progress_step_row: f32 = (height as f32).recip();

        for j in 0..height {
            current_progress += progress_step_row;
            {
                if let Ok(mut settings) = pt_ctx.settings.try_lock() {
                    settings.progress = current_progress;
                }
            }

            for i in 0..width {
                let col_vec3 = (0..self.samples_per_pixel)
                    .into_par_iter()
                    .map(|_s| {
                        let rnd = rand_in_square();
                        let ray = self.camera.get_ray(
                            (i as f32 + rnd.x) / width as f32,
                            (j as f32 + rnd.y) / height as f32,
                        );
                        self.scene.get_color(ray)
                    })
                    .reduce(|| Vec3::ZERO, |accum, color| accum + color)
                    * (self.samples_per_pixel as f32).recip();

                let col = Color::new(
                    border.clamp(col_vec3.x),
                    border.clamp(col_vec3.y),
                    border.clamp(col_vec3.z),
                    1f32,
                );

                image_data[(j * pt_ctx.result_width + i) as usize] = col;
            }
        }
        match pt_ctx.tx.try_send(image_data) {
            Ok(_) => {
                debug!("frame has sent");
            }
            Err(_) => {
                debug!("swapchain is full");
                sleep(Duration::from_millis(16));
            }
        }
    }
}
