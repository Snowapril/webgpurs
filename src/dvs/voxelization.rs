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
    bind_group_global: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
    voxel_projection_pipeline: wgpu::ComputePipeline,
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
        let device_context = device_context.borrow();
        let view_proj = self.camera.borrow().build_view_proj_matrix();
        device_context.queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice(view_proj.as_ref()),
        );
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
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bind_group_global, &[]);

        let num_scene_objects = self.scene_objects.len();
        for index in 0..num_scene_objects {
            rpass.set_bind_group(1, &self.bind_groups[index], &[]);
            rpass.set_index_buffer(
                self.scene_objects[index].index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            rpass.set_vertex_buffer(0, self.scene_objects[index].vertex_buffer.slice(..));
            rpass.draw_indexed(0..self.scene_objects[index].num_indices as u32, 0, 0..1);
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
        let voxel_axis_projection_shader =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("VoxelAxisProjection Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "../shader/voxel_axis_projection.wgsl"
                ))),
            });
        let voxelization_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxelization Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shader/voxelization.wgsl"
            ))),
        });

        let (projection_bind_group_layout, projection_pipeline) =
            Self::init_voxel_projection_pipeline(device, &shader)?;

        Ok(Self {
            camera,
            scene_objects: scene_objects_loaded,
            bind_groups,
            pipeline,
            bind_group_global,
            camera_uniform_buffer,
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
    ) -> Result<(wgpu::BindGroupLayout, wgpu::ComputePipeline)> {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Voxel Axis Projection BindGroupLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
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
                        min_binding_size: wgpu::BufferSize::new(12),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(12),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Voxel Axis Projection PipelineLayout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Voxel Axis Projection Pipeline"),
            layout: Some(&pipeline_layout),
            module: shader_module,
            entry_point: "voxel_projection_cs",
        });

        Ok((bind_group_layout, compute_pipeline))
    }
}
