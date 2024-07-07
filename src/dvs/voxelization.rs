//! Scene voxelization pass
//!
//! As wgpu does not support geometry shader, voxelization pass is composed of two step.
//! 1. Voxel Axis Projection Pass
//!     a. Project given each triangles into voxel axis to read-write buffer.
//! 2. Voxelization Pass
//!     a. Use read-write buffer generated from Voxel-Axis-Projection-Pass as vertex buffer
//!         for this primitive input.
//!     b. Store each attributes (albedo, normal, ..) to storage texture
//!

use crate::{
    pass::{black_board, render_context, render_pass},
    render_client::camera::Camera,
    render_device,
    scene::scene_object,
    shader_pipeline::shader,
};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{
    borrow::Cow,
    cell::Cell,
    cell::{Ref, RefCell, RefMut},
    f32::consts,
    mem,
    num::NonZeroU32,
    rc::Rc,
};
use wgpu::util::DeviceExt;

pub struct VoxelizationPass {
    camera: Rc<RefCell<Camera>>,
    scene_objects: Vec<scene_object::SceneObject>,
    projection_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    bind_group_per_mesh: wgpu::BindGroup,
}

impl render_pass::RenderPass for VoxelizationPass {
    fn process_event(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn update_render(
        &mut self,
        device_context: &RefCell<render_device::RenderDeviceContext>,
        _black_board: &RefMut<black_board::BlackBoard>,
    ) {
    }

    fn on_resized(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device_context: &RefCell<render_device::RenderDeviceContext>,
    ) {
    }

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device_context: &RefCell<render_device::RenderDeviceContext>,
        render_context: &Ref<render_context::RenderContext>,
        black_board: &RefMut<black_board::BlackBoard>,
    ) {
        let device_context = device_context.borrow();
        let mut encoder: wgpu::CommandEncoder = device_context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            cpass.set_pipeline(&self.projection_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_bind_group(1, &self.bind_group_per_mesh, &[]);
            cpass.dispatch_workgroups(64, 1, 1);
        }
    }
}

impl VoxelizationPass {
    pub(crate) fn create_pass(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        camera: Rc<RefCell<Camera>>,
        scene_objects_loaded: Vec<scene_object::SceneObject>,
    ) -> Result<Self> {
        let voxel_axis_projection_shader = shader::create_shader_module(
            device,
            include_str!("../shader/glsl/voxel_axis_projection.glsl"),
            "voxel_axis_projection.glsl",
            "main",
            shaderc::ShaderKind::Compute,
        )?;

        let voxelization_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxelization Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shader/voxelization.wgsl"
            ))),
        });

        let (
            projection_bind_group_layout,
            projection_bind_group_layout_per_mesh,
            projection_pipeline,
        ) = Self::init_voxel_projection_pipeline(device, &voxel_axis_projection_shader)?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VoxelAxisProjection BindGroup"),
            layout: &projection_bind_group_layout,
            entries: &[],
        });

        let bind_group_per_mesh = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("VoxelAxisProjection BindGroupPerMesh"),
            layout: &projection_bind_group_layout_per_mesh,
            entries: &[],
        });

        Ok(Self {
            camera,
            scene_objects: scene_objects_loaded,
            projection_pipeline,
            bind_group,
            bind_group_per_mesh,
        })
    }

    fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(1.5f32, -5.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );
        projection * view
    }

    /// Create voxel axis projection compute pipeline
    ///
    /// As webgpu don't have geometry shader, for projecting given vertices into voxel axis
    /// we use compute pass for projecting each vertices into uav and use it as vertex buffer for
    /// the next rasterization pass
    fn init_voxel_projection_pipeline(
        device: &wgpu::Device,
        shader_module: &wgpu::ShaderModule,
    ) -> Result<(
        wgpu::BindGroupLayout,
        wgpu::BindGroupLayout,
        wgpu::ComputePipeline,
    )> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Voxel Axis Projection BindGroupLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(256),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
            ],
        });

        let bind_group_layout_per_mesh =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Voxel Axis Projection BindGroupLayoutPerMesh"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(16),
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voxel Axis Projection PipelineLayout"),
            bind_group_layouts: &[&bind_group_layout, &bind_group_layout_per_mesh],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..32,
            }],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Voxel Axis Projection Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader_module,
            compilation_options: Default::default(),
            entry_point: "main",
        });

        Ok((
            bind_group_layout,
            bind_group_layout_per_mesh,
            compute_pipeline,
        ))
    }
}
