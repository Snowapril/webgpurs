use crate::{
    dvs::voxelization,
    pass::{black_board, render_context, render_pass},
    render_client::{camera::Camera, camera_controller::CameraController, render_device},
    scene::{self, scene_object_loader},
};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::collections::HashMap;
use std::{
    borrow::Cow,
    boxed::Box,
    cell::{Cell, RefCell},
    f32::consts,
    mem,
    num::NonZeroU32,
    rc::Rc,
};
use wgpu::util::DeviceExt;

#[derive(Parser)] // requires `derive` feature
#[command(author, version, about, long_about = None)]
struct CommandLineArguments {
    #[arg(short = 'i')]
    obj_path: String,
}
pub struct DeferredVoxelShading {
    passes: Vec<RefCell<Box<dyn render_pass::RenderPass>>>,
    camera: Rc<RefCell<Camera>>,
    camera_controller: CameraController,
    render_context: RefCell<render_context::RenderContext>,
    black_board: RefCell<black_board::BlackBoard>,
}

impl render_device::RenderDevice for DeferredVoxelShading {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::BUFFER_BINDING_ARRAY | 
        wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY |
        wgpu::Features::PUSH_CONSTANTS
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
            max_storage_buffers_per_shader_stage: 8,
            max_storage_textures_per_shader_stage: 4,
            max_uniform_buffers_per_shader_stage: 12,
            max_uniform_buffer_binding_size: 16 << 10,
            max_storage_buffer_binding_size: 128 << 20,
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_vertex_buffer_array_stride: 2048,
            max_push_constant_size: 32,
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
        config: &wgpu::SurfaceConfiguration,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) -> Result<Self> {
        let device_context = device_context.borrow();
        let args = CommandLineArguments::parse();
        let scene_objects =
            scene_object_loader::load_scene_objects(&device_context.device, &args.obj_path)?;
        let mut passes: Vec<RefCell<Box<dyn render_pass::RenderPass>>> = vec![];

        let camera = Rc::new(RefCell::new(Camera {
            eye: glam::Vec3::new(0.0, 1.0, 3.0),
            dir: glam::Vec3::new(0.0, 0.0, 1.0),
            aspect: config.width as f32 / config.height as f32,
            ..Default::default()
        }));
        let camera_controller = CameraController::new(0.05, camera.clone());

        let voxelization_pass = voxelization::VoxelizationPass::create_pass(
            config,
            &device_context.adapter,
            &device_context.device,
            &device_context.queue,
            camera.clone(),
            scene_objects,
        )?;
        passes.push(RefCell::new(Box::new(voxelization_pass)));

        Ok(DeferredVoxelShading {
            passes,
            camera,
            camera_controller,
            render_context: RefCell::new(render_context::RenderContext {}),
            black_board: RefCell::new(black_board::BlackBoard {
                textures: HashMap::default(),
            }),
        })
    }

    fn process_event(&mut self, event: winit::event::WindowEvent) {
        self.camera_controller.process_input(&event);

        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().process_event(event.clone());
        })
    }

    fn update_render(&mut self, device_context: &RefCell<render_device::RenderDeviceContext>) {
        self.camera_controller.update_camera(0.0);
        self.passes.iter().for_each(|pass| {
            pass.borrow_mut()
                .update_render(&device_context, &self.black_board.borrow_mut());
        })
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) {
        self.camera.borrow_mut().aspect = config.width as f32 / config.height as f32;
        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().on_resized(config, &device_context);
        })
    }

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    ) {
        let mut encoder: wgpu::CommandEncoder = device_context
            .borrow()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().render(
                back_buffer_view,
                &mut encoder,
                &device_context,
                &self.render_context.borrow(),
                &self.black_board.borrow_mut(),
            );
        });

        device_context.borrow().queue.submit(Some(encoder.finish()));
    }
}
