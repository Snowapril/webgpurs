use crate::{
    pass::{render_context, render_pass},
    scene::scene_object,
};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{borrow::Cow, cell::Cell, cell::RefCell, f32::consts, mem, num::NonZeroU32, rc::Rc};
use wgpu::util::DeviceExt;

pub struct VoxelizationPass {
    scene_objects: Vec<scene_object::SceneObject>,
    bind_groups: Vec<wgpu::BindGroup>,
    pipeline: wgpu::RenderPipeline,
}

impl render_pass::RenderPass for VoxelizationPass {
    fn process_event(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn update_render(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn on_resized(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn render(
        &mut self,
        _back_buffer_view: &wgpu::TextureView,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }
}

impl VoxelizationPass {
    pub(crate) fn create_pass(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
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
                    layout: &bind_group_layout,
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
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(Self {
            scene_objects: scene_objects_loaded,
            bind_groups,
            pipeline,
        })
    }
}
