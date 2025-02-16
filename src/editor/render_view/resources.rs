use crate::editor::settings::Settings;
use bytemuck::{Pod, Zeroable};
use eframe::wgpu;
use std::sync::{Arc, Mutex};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Uniforms {
    color: [f32; 4],
    camera_to_world: [[f32; 4]; 4],
    light_dir: [f32; 4],
    light_col: [f32; 4],
    absorption: f32,
    scattering: f32,
    g: f32,
    step_size: f32,
    //samples_per_pixel: u32,
}

pub struct FullScreenTriangleRenderResources {
    pub blit_pipeline: wgpu::RenderPipeline,
    pub blit_bind_group: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline,
    pub compute_bind_group: wgpu::BindGroup,

    pub settings: Arc<Mutex<Settings>>,
    pub uniforms_buffer: wgpu::Buffer,
}

impl FullScreenTriangleRenderResources {
    pub fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Ok(settings) = self.settings.lock() {
            let color = settings.background_color;
            let camera_to_world = settings.matrix;
            let light_color = settings.light_color * settings.lightness;
            let uniforms = Uniforms {
                color: [color[0], color[1], color[2], 1f32],
                camera_to_world: camera_to_world.to_cols_array_2d(),
                g: settings.g,
                light_col: [light_color.x, light_color.y, light_color.z, 1.0],
                light_dir: [1.0, 1.0, 1.0, 1.0],
                absorption: settings.absorption,
                scattering: settings.scattering,
                step_size: settings.ray_marching_step,
                //samples_per_pixel: settings.spp,
            };

            queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));
        }
    }

    pub fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.blit_pipeline);
        render_pass.set_bind_group(0, &self.blit_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
