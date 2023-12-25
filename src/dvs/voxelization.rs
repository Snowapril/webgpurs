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
    bind_groups: Vec<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
    bind_group_global: wgpu::BindGroup,
    camera_uniform_buffer: wgpu::Buffer,
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
        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        // Create other resources
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let camera_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group
        let bind_group_global = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let bind_group_layout_per_pass =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(mem::size_of::<
                            scene_object::MaterialPod,
                        >() as u64),
                    },
                    count: None,
                }],
            });

        let bind_groups = scene_objects_loaded
            .iter()
            .map(|scene_object| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &bind_group_layout_per_pass,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: scene_object.material.as_entire_binding(),
                    }],
                    label: Some(format!("Material Buffer [ {} ]", scene_object.name).as_str()),
                })
            })
            .collect::<Vec<wgpu::BindGroup>>();

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &bind_group_layout_per_pass],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shader/voxelization.wgsl"
            ))),
        });

        let input_layout = [wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<scene_object::VertexPod>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &input_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.view_formats[0].into())],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

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
}
