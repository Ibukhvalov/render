use crossbeam_channel::Sender;
use tracing::{debug, error};

extern crate nalgebra_glm as glm;

use glam::Vec3 as glamVec3;
use super::scene::Scene;
use super::scene::camera::Camera;
use crate::interval::Interval;
use crate::util::*;

use crate::Settings;
use glm::{Mat4, Vec3};
use log::info;
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use egui_wgpu::wgpu::util::RenderEncoder;
use rayon::prelude::*;

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
    view: Mat4,
    tx: Sender<Vec<Color>>,
    input_rx: single_value_channel::Receiver<Mat4>,
    settings: Arc<Mutex<Settings>>,
}
impl PathTracerRenderContext {
    pub fn new(
        width: u32,
        height: u32,
        // scene: Arc<EmbreeScene>,
        tx: Sender<Vec<Color>>,
        input_rx: single_value_channel::Receiver<Mat4>,
        settings: Arc<Mutex<Settings>>,
    ) -> Self {
        Self {
            result_height: height,
            result_width: width,
            view: Mat4::new_translation(&Vec3::new(0.0f32, 0.0f32, -1.0f32)),
            // scene,
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
}

impl Renderer {
    pub fn new() -> Self {
        let camera = Camera::new(glamVec3::new(0.,0.,100.), glamVec3::ZERO, glamVec3::Y, 80., 1.0);
        Self {
            camera,
            scene: Scene::new(glamVec3::new(0.6,0.6,0.9)),
        }
    }
}

impl Renderer {
    pub fn run_iteration(&mut self, pt_ctx: &mut PathTracerRenderContext) {
        let camera_matrix = pt_ctx.input_rx.latest();
        //info!("camera matrix: {}", camera_matrix);


        // copy all settings here
        if let Ok(settings) = pt_ctx.settings.lock() {
            self.scene.change_background(glamVec3::new(settings.color[0], settings.color[1], settings.color[2]));
        } else {
            error!("Could not acquire settings lock, skipping this frame.");
        }


        let samples_per_pixel = 3;
        let border = Interval::new(0., 0.9999);


        let mut image_data = pt_ctx.image_data.lock().unwrap().clone();

        let (width, height) = (pt_ctx.result_width, pt_ctx.result_height);

        info!("update {width} {height}");

        let image_aspect = width as f32 / height as f32;
        self.camera.update(80., image_aspect);

        for j in 0..height {
            for i in 0..width {
                let col_vec3 = (0..samples_per_pixel)
                    .into_par_iter()
                    .map(|_s| {
                        let rnd = rand_in_square();
                        let ray = self.camera.get_ray(
                            (i as f32 + rnd.x) / width as f32,
                            (j as f32 + rnd.y) / height as f32,
                        );
                        self.scene.get_color(ray)
                    })
                    .reduce(|| glamVec3::ZERO, |accum, color| accum + color)
                    * (samples_per_pixel as f32).recip();

                let col = Color::new(border.clamp(col_vec3.x), border.clamp(col_vec3.y), border.clamp(col_vec3.z), 1f32);
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