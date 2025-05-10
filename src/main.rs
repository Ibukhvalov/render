mod aabb;
mod editor;
mod volume_grid;

use editor::Editor;
use eframe::wgpu;

use std::sync::Arc;

const SCREEN_SIZE: [u32; 2] = [800u32, 600u32];
const CAMERA_MOVE_SPEED: f32 = 2f32;
const CAMERA_ROTATION_SPEED: f32 = 2f32;
const WORKGROUP_SIZE: [u32; 2] = [16u32, 16u32];

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();

    let options = egui_wgpu::WgpuConfiguration {
        device_descriptor: Arc::new(|adapter| {
            let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            };

            wgpu::DeviceDescriptor {
                label: Some("egui wgpu device"),
                required_features: wgpu::Features::FLOAT32_FILTERABLE,
                memory_hints: wgpu::MemoryHints::Performance,
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 200000000,
                    // When using a depth buffer, we have to be able to create a texture
                    // large enough for the entire surface, and we want to support 4k+ displays.
                    max_texture_dimension_2d: 8192,
                    ..base_limits
                },
            }
        }),
        ..Default::default()
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([SCREEN_SIZE[0] as f32, SCREEN_SIZE[1] as f32]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: options,
        ..Default::default()
    };

    eframe::run_native(
        "Strelka",
        options,
        Box::new(|cc| Ok(Box::new(Editor::new(cc, SCREEN_SIZE[0], SCREEN_SIZE[1])))),
    )
}
