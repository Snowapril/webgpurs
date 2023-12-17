use crate::{
    dvs::voxelization,
    pass::render_pass,
    render_client::{camera::Camera, camera_controller::CameraController, render_device},
    scene::{self, scene_object_loader},
};
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
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
}

impl render_device::RenderDevice for DeferredVoxelShading {
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
        config: &wgpu::SurfaceConfiguration,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self> {
        let args = CommandLineArguments::parse();
        let scene_objects = scene_object_loader::load_scene_objects(device, &args.obj_path)?;
        let mut passes: Vec<RefCell<Box<dyn render_pass::RenderPass>>> = vec![];

        let voxelization_pass = voxelization::VoxelizationPass::create_pass(
            config,
            adapter,
            device,
            queue,
            scene_objects,
        )?;
        passes.push(RefCell::new(Box::new(voxelization_pass)));

        let camera = Rc::new(RefCell::new(Camera {
            eye: glam::Vec3::new(0.0, 0.0, -3.0),
            aspect: config.width as f32 / config.height as f32,
            ..Default::default()
        }));
        let camera_controller = CameraController::new(0.01, camera.clone());

        Ok(DeferredVoxelShading {
            passes,
            camera,
            camera_controller,
        })
    }

    fn process_event(&mut self, event: winit::event::WindowEvent) {
        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().process_event(event.clone());
        })
    }

    fn update_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(0.0);

        let view_proj = self.camera.borrow().build_view_proj_matrix();
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(view_proj.as_ref()),
        );

        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().update_render(device, queue);
        })
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.camera.borrow_mut().aspect = config.width as f32 / config.height as f32;

        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().on_resized(config, device, queue);
        })
    }

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.passes.iter().for_each(|pass| {
            pass.borrow_mut().render(back_buffer_view, device, queue);
        })
    }
}
