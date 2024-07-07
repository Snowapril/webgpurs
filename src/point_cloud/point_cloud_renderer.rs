use crate::{
    point_cloud::point_cloud::{Batch, PointCloud},
    render_client::render_device,
};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    f32::consts,
    mem,
    num::NonZeroU32,
};
use wgpu::util::DeviceExt;

#[derive(Parser)] // requires `derive` feature
#[command(author, version, about, long_about = None)]
struct CommandLineArguments {
    #[arg(short = 'i')]
    e57_path: String,
}

pub struct PointCloudRenderer {
    point_cloud: Cell<PointCloud>,
    bind_group_global: wgpu::BindGroup,
    bind_group_per_pass: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
}

impl render_device::RenderDevice for PointCloudRenderer {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::BUFFER_BINDING_ARRAY | wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY
    }

    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
        }
    }

    fn required_limits() -> wgpu::Limits {
        // same for wgpu::Limits::downlevel_defaults()
        wgpu::Limits {
            max_texture_dimension_1d: 2048,
            max_texture_dimension_2d: 2048,
            max_texture_dimension_3d: 256,
            max_texture_array_layers: 256,
            max_bind_groups: 4,
            max_bindings_per_bind_group: 1000,
            max_dynamic_uniform_buffers_per_pipeline_layout: 8,
            max_dynamic_storage_buffers_per_pipeline_layout: 4,
            max_sampled_textures_per_shader_stage: 16,
            max_samplers_per_shader_stage: 16,
            max_storage_buffers_per_shader_stage: 4,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_uniform_buffer_binding_size: 16 << 10,
            max_storage_buffer_binding_size: 128 << 20,
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            max_push_constant_size: 0,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 256,
            max_inter_stage_shader_components: 60,
            max_compute_workgroup_storage_size: 16352,
            max_compute_invocations_per_workgroup: 256,
            max_compute_workgroup_size_x: 256,
            max_compute_workgroup_size_y: 256,
            max_compute_workgroup_size_z: 64,
            max_compute_workgroups_per_dimension: 65535,
            max_buffer_size: 256 << 20,
            max_non_sampler_bindings: 1_000_000,
            ..Default::default()
        }
    }

    fn init(
        _config: &wgpu::SurfaceConfiguration,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) -> Result<Self> {
        let device_context = device_context.borrow();
        let args = CommandLineArguments::parse();
        let point_cloud = Cell::new(PointCloud::from(&args.e57_path));

        let bind_group_layout_global =
            device_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    mem::size_of::<glam::Mat4>() as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    (mem::size_of::<u32>() * 2usize) as _,
                                ),
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group_layout_per_pass =
            device_context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    mem::size_of::<glam::Vec3> as _,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    mem::size_of::<glam::Vec3> as _,
                                ),
                            },
                            count: None,
                        },
                    ],
                });

        let bind_group_global =
            device_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout_global,
                    entries: &[
                //wgpu::BindGroupEntry {
                //    binding: 0,
                //    resource: uniform_buf.as_entire_binding(),
                //},
                //wgpu::BindGroupEntry {
                //    binding: 1,
                //    resource: wgpu::BindingResource::TextureView(&texture_view),
                //},
            ],
                    label: None,
                });

        let bind_group_per_pass =
            device_context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout_per_pass,
                    entries: &[
                //wgpu::BindGroupEntry {
                //    binding: 0,
                //    resource: uniform_buf.as_entire_binding(),
                //},
            ],
                    label: None,
                });

        let pipeline_layout =
            device_context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout_global, &bind_group_layout_per_pass],
                    push_constant_ranges: &[],
                });

        let shader = device_context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../shader/render_point_cs.wgsl"
                ))),
            });

        let pipeline =
            device_context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: "render_point_cs",
                    compilation_options: Default::default(),
                });

        Ok(PointCloudRenderer {
            point_cloud,
            bind_group_global,
            bind_group_per_pass,
            pipeline,
        })
    }

    fn process_event(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn update_render(&mut self, device_context: &RefCell<render_device::RenderDeviceContext>) {}

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) {
    }

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) {
        let device_context = device_context.borrow();
        let mut encoder = device_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: back_buffer_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        device_context.queue.submit(Some(encoder.finish()));
    }
}
