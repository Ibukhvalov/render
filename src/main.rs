//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashSet;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use eframe::egui_wgpu::{self, wgpu};
use egui::{InputState, Key, Rect};
use glam::{BVec3A, Mat4, Quat, Vec3, Vec4};

use crate::render::Renderer;
use crossbeam_channel::{Receiver, Sender};
use egui::Key::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp};
use render::Color;
use render::PathTracerRenderContext;

mod interval;
mod render;
mod scene;
mod util;

struct RenderView {}

#[derive(Clone)]
struct RenderViewCallback {
    // TODO: rework with arc mutex?
    receiver: Arc<Receiver<Vec<Color>>>,
}

impl egui_wgpu::CallbackTrait for RenderViewCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &FullScreenTriangleRenderResources = resources.get().unwrap();

        if let Ok(image) = self.receiver.try_recv() {
            queue.write_buffer(
                &resources.staging_buffer,
                0,
                bytemuck::cast_slice(image.as_slice()),
            );
            let tex_width = resources.result_texture.size().width as usize;
            egui_encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &resources.staging_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((tex_width * size_of::<glam::Vec4>()) as u32),
                        rows_per_image: None,
                    },
                },
                resources.result_texture.as_image_copy(),
                resources.result_texture.size(),
            );
        }

        resources.prepare(device, queue); // TODO: pass screen dims here
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &FullScreenTriangleRenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct FullScreenTriangleRenderResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    staging_buffer: wgpu::Buffer,
    result_texture: wgpu::Texture,
}

impl FullScreenTriangleRenderResources {
    fn prepare(&self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // Update our uniform buffer with the angle from the UI
        // queue.write_buffer(
        //     &self.uniform_buffer,
        //     0,
        //     bytemuck::cast_slice(&[angle, 0.0, 0.0, 0.0]),
        // );
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

impl RenderView {
    // TODO: setup wgpu pipeline for presenting full screen triangle with texture
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, width: u32, height: u32) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;

        let device = &wgpu_render_state.device;

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let staging_buffer_size: usize = (width * height) as usize * std::mem::size_of::<Vec4>();

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            size: staging_buffer_size as u64,
            mapped_at_creation: false,
        });

        let result_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Result texture"),
            view_formats: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./blit.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let result_texture_view =
            result_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let result_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let textures_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&result_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&result_sampler),
                },
            ],
            label: Some("textures_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu_render_state.target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(FullScreenTriangleRenderResources {
                pipeline: render_pipeline,
                bind_group: textures_bind_group,
                staging_buffer,
                result_texture,
            });

        Some(Self {})
    }
}

#[derive(Debug)]
struct Settings {
    background_color: Vec3,
    light_color: Vec3,
    g: f32,
    absorption: f32,
    scattering: f32,
    spp: f32,
    progress: f32,
    ray_marching_step: f32,
    picked_path: Option<String>,
}

#[derive(Debug)]
enum PaneType {
    Settings(Arc<Mutex<Settings>>),
    Render(Arc<Receiver<Vec<Color>>>),
}

#[derive(Debug)]
struct Pane {
    nr: usize,
    kind: PaneType,
}

struct TreeBehavior {}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            // self.add_child_to = Some(tile_id);
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match &pane.kind {
            PaneType::Settings(settings) => {
                ui.label("Settings.".to_string());
                // let color = settings.as_mut_slice();
                // if let Some(color_array) = color.get_mut(0..3) {
                //     ui.color_edit_button_rgb(color_array.try_into().unwrap());
                // }
                // Acquire a lock to modify settings
                if let Ok(mut settings) = settings.lock() {
                    ui.color_edit_button_rgb(settings.background_color.as_mut());
                    ui.label("background color");
                    ui.color_edit_button_rgb(settings.light_color.as_mut());
                    ui.label("light color");
                    ui.add(egui::Slider::new(&mut settings.g, -1.0..=1.0).text("g"));
                    ui.add(
                        egui::Slider::new(&mut settings.absorption, 0.0..=1.5).text("absorption"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.scattering, 0.0..=2.0).text("scattering"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.spp, 1.0..=50.0).text("samples per pixel"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.ray_marching_step, 1.0..=10.0)
                            .text("ray marching step"),
                    );
                    ui.add(egui::ProgressBar::new(settings.progress).desired_width(200.0));
                } else {
                    ui.label("Failed to acquire settings lock.");
                }
                // ui.color_edit_button_rgb(color);
            }
            PaneType::Render(rx) => {
                // ui.checkbox(&mut self.checked, "Checked");
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    // self.viewport.ui(ui);
                    // let color = egui::epaint::Hsva::new(0.103 * pane.nr as f32, 0.5, 0.5, 1.0);
                    // ui.painter().rect_filled(ui.max_rect(), 0.0, color);

                    // let rect = ui.max_rect();
                    // let response = ui.allocate_rect(rect, egui::Sense::drag());

                    let width = ui.max_rect().width();
                    let heigth = ui.max_rect().height();
                    let (rect, response) = ui.allocate_at_least(
                        egui::Vec2::new(width, heigth - 20.0f32),
                        egui::Sense::drag(),
                    );

                    // TODO: pass input to camera controller
                    if response.has_focus() {}

                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderViewCallback {
                            receiver: rx.clone(),
                        },
                    ));
                });
            }
        }

        // You can make your pane draggable like so:
        if ui
            .add(egui::Button::new("Drag me!").sense(egui::Sense::drag()))
            .drag_started()
        {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

struct Editor {
    viewport: Option<RenderView>,
    tree: egui_tiles::Tree<Pane>,
    input_tx: single_value_channel::Updater<Mat4>,
    settings: Arc<Mutex<Settings>>,
    camera_to_world: View,
}

struct View {
    rotation_x: Quat,
    rotation_y: Quat,
    translation: Vec3,
}

impl View {
    pub fn default() -> Self {
        Self {
            rotation_x: Quat::IDENTITY,
            rotation_y: Quat::IDENTITY,
            translation: Vec3::NEG_Z * 120f32,
        }
    }
}

impl Editor {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        width: u32,
        height: u32,
        rx: Receiver<Vec<Color>>,
        input_tx: single_value_channel::Updater<Mat4>,
        settings: Arc<Mutex<Settings>>,
    ) -> Self {
        catppuccin_egui::set_theme(&_cc.egui_ctx, catppuccin_egui::MOCHA);
        let tree = create_tree(rx, settings.clone());
        Self {
            viewport: RenderView::new(_cc, width, height),
            tree,
            input_tx,
            settings,
            camera_to_world: View::default(),
        }
    }
}

impl Editor {
    fn handle_key_down(&mut self, keys: HashSet<Key>) {
        let rotation = self.camera_to_world.rotation_y * self.camera_to_world.rotation_x;

        let view_dir = (rotation * Vec3::Z).normalize();
        let right_dir = (rotation * Vec3::X).normalize();
        let up_dir = (Vec3::Y).normalize();

        if keys.contains(&Key::W) {
            self.camera_to_world.translation += view_dir;
        }
        if keys.contains(&Key::S) {
            self.camera_to_world.translation -= view_dir;
        }
        if keys.contains(&Key::A) {
            self.camera_to_world.translation -= right_dir;
        }
        if keys.contains(&Key::D) {
            self.camera_to_world.translation += right_dir;
        }
        if keys.contains(&Key::Q) {
            self.camera_to_world.translation -= up_dir;
        }
        if keys.contains(&Key::E) {
            self.camera_to_world.translation += up_dir;
        }

        if keys.contains(&ArrowDown) {
            self.camera_to_world.rotation_x *= Quat::from_rotation_x(0.01f32);
        }
        if keys.contains(&ArrowUp) {
            self.camera_to_world.rotation_x *= Quat::from_rotation_x(-0.01f32);
        }
        if keys.contains(&ArrowRight) {
            self.camera_to_world.rotation_y *= Quat::from_rotation_y(0.01f32);
        }
        if keys.contains(&ArrowLeft) {
            self.camera_to_world.rotation_y *= Quat::from_rotation_y(-0.01f32);
        }

        self.send_camera_matrix();
    }

    fn handle_mouse(&mut self, pointer: egui::PointerState) {
        //self.camera_to_world.rotation_x *= Quat::from_rotation_y(pointer.delta().x * 0.002);
        //self.camera_to_world.rotation_y *= Quat::from_rotation_x(pointer.delta().y * 0.002);
    }

    fn send_camera_matrix(&self) {
        let rotation = self.camera_to_world.rotation_y * self.camera_to_world.rotation_x;
        self.input_tx
            .update(Mat4::from_rotation_translation(
                rotation,
                self.camera_to_world.translation,
            ))
            .expect("Cant send matrix");
    }
}

impl eframe::App for Editor {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let input = ctx.input(|i| i.clone());
        self.handle_key_down(input.keys_down);
        self.handle_mouse(input.pointer);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Ok(mut settings) = self.settings.lock() {
                                settings.picked_path = Some(path.display().to_string());
                            }
                        }
                    }

                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
        // egui::SidePanel::left("tree").show(ctx, |ui| {
        //     ui.collapsing("Tree", |ui| {
        //         let tree_debug = format!("{:#?}", self.tree);
        //         ui.monospace(&tree_debug);
        //     });

        //     ui.separator();
        // });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            self.tree.ui(&mut behavior, ui);
        });

        // TODO: high cpu usage here we need to repaint only render viewport
        ctx.request_repaint();
    }
}

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
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: options,
        ..Default::default()
    };

    let (matrix_receiver, matrix_updater) =
        single_value_channel::channel_starting_with(Mat4::IDENTITY);
    let (render_result_tx, render_result_rx): (Sender<Vec<Color>>, Receiver<Vec<Color>>) =
        crossbeam_channel::bounded(3);

    let settings: Settings = Settings {
        background_color: Vec3::new(0.7f32, 0.7f32, 0.9f32),
        light_color: Vec3::new(1.0, 0.9, 0.9),
        g: 0.6,
        absorption: 0.13,
        scattering: 0.8,
        ray_marching_step: 10f32,
        spp: 1f32,
        progress: 0f32,
        picked_path: None,
    };
    let settings = Arc::new(Mutex::new(settings));

    let path_tracer_render_lock = Arc::new(RwLock::new(PathTracerRenderContext::new(
        256,
        256,
        render_result_tx.clone(),
        matrix_receiver,
        settings.clone(),
    )));
    let pt_render = path_tracer_render_lock.clone();

    let mut renderer: Renderer = Renderer::new();

    thread::spawn(move || loop {
        if let Ok(mut p) = pt_render.write() {
            renderer.run_iteration(&mut p);
        }
    });

    eframe::run_native(
        "Strelka",
        options,
        Box::new(|cc| {
            Ok(Box::new(Editor::new(
                cc,
                256,
                256,
                render_result_rx,
                matrix_updater,
                settings,
            )))
        }),
    )
}

fn create_tree(
    render_result_rx: Receiver<Vec<Color>>,
    settings: Arc<Mutex<Settings>>,
) -> egui_tiles::Tree<Pane> {
    let mut next_view_nr = 0;
    let gen_pane = || {
        let pane = Pane {
            nr: next_view_nr,
            kind: PaneType::Settings(settings),
        };
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];

    let render_pane = Pane {
        nr: 0,
        kind: PaneType::Render(Arc::new(render_result_rx)),
    };
    tabs.push(tiles.insert_pane(render_pane));

    tabs.push(tiles.insert_pane(gen_pane()));

    // let root = tiles.insert_tab_tile(tabs);
    let root = tiles.insert_horizontal_tile(tabs);

    egui_tiles::Tree::new("strelka_tree", root, tiles)
}
