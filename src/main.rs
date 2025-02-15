//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

use eframe::egui_wgpu::{self, wgpu};
use eframe::wgpu::util::DeviceExt;
use eframe::wgpu::{include_wgsl, BufferUsages, ComputePassDescriptor};
use egui::{Key};
use glam::{Mat4, Quat, Vec3};
use bytemuck::{Pod, Zeroable};

const SCREEN_SIZE: [u32; 2] = [800u32, 600u32];

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct Uniforms {
    color: [f32; 4],
    camera_to_world: [[f32;4]; 4],
    light_dir: [f32; 4],
    light_col: [f32; 4],
    absorption: f32,
    scattering: f32,
    g: f32,
    step_size: f32,
    //samples_per_pixel: u16,
}

/*color: vec4f,
    camera_to_world: mat4x4f,
    light_dir: vec3f,
    light_col: vec3f,
    absorption: f32,
    scattering: f32,
    g: f32,
    step_size: f32, */
    
mod aabb;
mod volume_grid;

use log::debug;
use volume_grid::VolumeGridStatic;

struct RenderView {}

#[derive(Clone)]
struct RenderViewCallback {}

impl egui_wgpu::CallbackTrait for RenderViewCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &FullScreenTriangleRenderResources = resources.get().unwrap();
        
        {
            let mut compute_pass = egui_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&resources.compute_pipeline);
            compute_pass.set_bind_group(0, &resources.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(SCREEN_SIZE[0], SCREEN_SIZE[1], 1);
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
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,

    settings: Arc<Mutex<Settings>>,
    uniforms_buffer: wgpu::Buffer
}

impl FullScreenTriangleRenderResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Ok(settings) = self.settings.lock() {
            let color = settings.background_color;
            let camera_to_world = settings.matrix;
            let light_color = settings.light_color * settings.lightness;
            let uniforms = Uniforms {
                color: [color[0], color[1], color[2], 1f32],
                camera_to_world: camera_to_world.to_cols_array_2d(),
                g: settings.g,
                light_col: [light_color.x, light_color.y, light_color.z, 1.0],
                light_dir: [1.0,1.0,1.0, 1.0],
                absorption: settings.absorption,
                scattering: settings.scattering,
                step_size: settings.ray_marching_step,
                //samples_per_pixel: settings.spp,
            };

            queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::bytes_of(&uniforms));

        }

    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.blit_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
    }
}

impl RenderView {
    // TODO: setup wgpu pipeline for presenting full screen triangle with texture
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, width: u32, height: u32, settings: Arc<Mutex<Settings>>) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;


        let filename = std::env::args()
            .nth(1)
            .expect("Missing VDB filename as first argument");
        //let filename = String::from("./data/vdbAssets/wdas_cloud_sixteenth.vdb");
        

        let mut vdb_reader = vdb_rs::VdbReader::new(BufReader::new(File::open(filename).unwrap())).unwrap();
        let grid_to_load = vdb_reader
            .available_grids()
            .first()
            .cloned()
            .unwrap_or(String::new());

        let (grid_static, weights) = VolumeGridStatic::build_from_vdb_grid(
            vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap());


        let device = &wgpu_render_state.device;

        let blit_module = device.create_shader_module(include_wgsl!("blit.wgsl"));
        let cs_module = device.create_shader_module(include_wgsl!("compute.wgsl"));


        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let result_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Result texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let result_texture_view = result_texture.create_view(&wgpu::TextureViewDescriptor::default());

        
        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });


        let volume_grid_static_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Volume grid buffer"),
            contents: bytemuck::bytes_of(&grid_static),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let weights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Weights buffer"),
            contents: bytemuck::cast_slice(weights.as_slice()),
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });



        let result_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        

        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Blit bind group layout"),
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

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group blit"),
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&result_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&result_sampler),
                }
            ],
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group compute"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&result_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(volume_grid_static_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(weights_buffer.as_entire_buffer_binding()),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(uniforms_buffer.as_entire_buffer_binding()),
                },
            ],
        });
 

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute pipeline layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let blit_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Blit pipeline layout"),
                bind_group_layouts: &[&blit_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &cs_module,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });


        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_module,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_module,
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
                blit_pipeline,
                blit_bind_group,
                compute_pipeline,
                compute_bind_group,
                uniforms_buffer,
                settings,
            });

        Some(Self {})
    }
}

#[derive(Debug)]
struct Settings {
    background_color: Vec3,
    light_color: Vec3,
    lightness: f32,
    g: f32,
    absorption: f32,
    scattering: f32,
    //spp: u16,
    ray_marching_step: f32,
    picked_path: Option<String>,
    matrix: Mat4,
}

impl Settings {
    fn default() -> Self {
        Self {
                background_color: Vec3::new(0.7f32, 0.7f32, 0.9f32),
                light_color: Vec3::new(1.0, 0.9, 0.9),
                lightness: 10f32,
                g: 0.6,
                absorption: 0.02,
                scattering: 0.4,
                ray_marching_step: 10f32,
                //spp: 1u16,
                picked_path: None,
                matrix: Mat4::IDENTITY,
        }
    }
}

#[derive(Debug)]
enum PaneType {
    Settings(Arc<Mutex<Settings>>),
    Render(()),
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
                        egui::Slider::new(&mut settings.absorption, 0.0..=0.1).text("absorption"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.scattering, 0.0..=0.5).text("scattering"),
                    );
                    ui.add(
                       egui::Slider::new(&mut settings.lightness, 5.0..=20.0).text("lightness"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.ray_marching_step, 1.0..=10.0)
                            .text("ray marching step"),
                    );
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

                    let width = SCREEN_SIZE[0] as f32;
                    let height = SCREEN_SIZE[1] as f32;

                    let (rect, response) = ui.allocate_at_least(
                        //egui::Vec2::new(width, heigth - 20.0f32),
                        egui::Vec2::new(width, height),
                        egui::Sense::drag(),
                    );

                    // TODO: pass input to camera controller
                    response.has_focus();

                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderViewCallback {
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
            translation: Vec3::NEG_Z * 150f32,
        }
    }
}

impl Editor {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        width: u32,
        height: u32,
    ) -> Self {
        catppuccin_egui::set_theme(&_cc.egui_ctx, catppuccin_egui::MOCHA);

        let settings = Settings::default();
    
        let settings = Arc::new(Mutex::new(settings));

        let tree = create_tree(settings.clone());
        Self {
            viewport: RenderView::new(_cc, width, height, settings.clone()),
            tree,
            settings: settings.clone(),
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
        
        for key in keys {
            self.camera_to_world.translation += match key {
                Key::W => view_dir,
                Key::S => -view_dir,
                Key::A => -right_dir,
                Key::D => right_dir,
                Key::Q => -up_dir,
                Key::E => up_dir,
                _ => Vec3::ZERO,
            };
            
            self.camera_to_world.rotation_x *= match key {
                Key::ArrowUp => Quat::from_rotation_x(-0.01f32),
                Key::ArrowDown => Quat::from_rotation_x(0.01f32),
                _ => Quat::IDENTITY,
            };

            self.camera_to_world.rotation_y *= match key {
                Key::ArrowRight => Quat::from_rotation_y(0.01f32),
                Key::ArrowLeft => Quat::from_rotation_y(-0.01f32),
                _ => Quat::IDENTITY,
            };
                
            }
        
        self.send_camera_matrix();
        
    }
 

    

    fn handle_mouse(&mut self, pointer: egui::PointerState) {
        //self.camera_to_world.rotation_x *= Quat::from_rotation_y(pointer.delta().x * 0.002);
        //self.camera_to_world.rotation_y *= Quat::from_rotation_x(pointer.delta().y * 0.002);
    }

    fn send_camera_matrix(&self) {
        let rotation = self.camera_to_world.rotation_y * self.camera_to_world.rotation_x;
        if let Ok(mut settings) = self.settings.lock() {
            settings.matrix = Mat4::from_rotation_translation(
                rotation,
                self.camera_to_world.translation)
        }
    }
}

impl eframe::App for Editor {
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

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
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
        viewport: egui::ViewportBuilder::default().with_inner_size([SCREEN_SIZE[0] as f32, SCREEN_SIZE[1] as f32]),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: options,
        ..Default::default()
    };

    

    


    eframe::run_native(
        "Strelka",
        options,
        Box::new(|cc| {
            Ok(Box::new(Editor::new(
                cc,
                SCREEN_SIZE[0],
                SCREEN_SIZE[1],
            )))
        }),
    )
}

fn create_tree(
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
        kind: PaneType::Render(()),
    };
    tabs.push(tiles.insert_pane(render_pane));

    tabs.push(tiles.insert_pane(gen_pane()));


   // let root = tiles.insert_tab_tile(tabs);
    let root = tiles.insert_horizontal_tile(tabs);

    egui_tiles::Tree::new("strelka_tree", root, tiles)    
}
