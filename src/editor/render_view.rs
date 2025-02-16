mod resources;

use resources::*;
use eframe::wgpu::{self, BufferUsages, util::DeviceExt};
use std::io::BufReader;
use std::fs::File;
use crate::volume_grid::VolumeGridStatic;
use std::sync::{Arc, Mutex};
use super::settings::Settings;
use eframe::wgpu::include_wgsl;

use crate::{SCREEN_SIZE, WORKGROUP_SIZE};

pub struct RenderView {}

#[derive(Clone)]
pub struct RenderViewCallback {}

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
        
        {
            let mut compute_pass = egui_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&resources.compute_pipeline);
            compute_pass.set_bind_group(0, &resources.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(SCREEN_SIZE[0]/WORKGROUP_SIZE[0], SCREEN_SIZE[1]/WORKGROUP_SIZE[1], 1);
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

impl RenderView {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, width: u32, height: u32, settings: Arc<Mutex<Settings>>) -> Option<Self> {
        let wgpu_render_state = cc.wgpu_render_state.as_ref()?;


        let filename = std::env::args()
            .nth(1)
            .expect("Missing VDB filename as first argument");
        

        let mut vdb_reader = vdb_rs::VdbReader::new(BufReader::new(File::open(filename).unwrap())).unwrap();
        let grid_to_load = vdb_reader
            .available_grids()
            .first()
            .cloned()
            .unwrap_or(String::new());

        let (grid_static, weights) = VolumeGridStatic::build_from_vdb_grid(
            vdb_reader.read_grid::<half::f16>(&grid_to_load).unwrap());


        let device = &wgpu_render_state.device;

        let blit_module = device.create_shader_module(include_wgsl!("../shaders/blit.wgsl"));
        let cs_module = device.create_shader_module(include_wgsl!("../shaders/compute.wgsl"));


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