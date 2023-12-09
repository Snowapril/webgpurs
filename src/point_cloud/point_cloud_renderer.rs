use crate::{
    point_cloud::point_cloud::{Batch, PointCloud},
    render_client::render_device,
};
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{borrow::Cow, cell::Cell, f32::consts, mem};
use wgpu::util::DeviceExt;

#[derive(Parser)] // requires `derive` feature
#[command(author, version, about, long_about = None)]
struct CommandLineArguments {
    #[arg(short = 'i')]
    e57_path: String,
}

pub struct PointCloudRenderer {
    point_cloud: Cell<PointCloud>,
}

impl render_device::RenderDevice for PointCloudRenderer {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::POLYGON_MODE_LINE
    }

    fn init(
        _config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        let args = CommandLineArguments::parse();
        let point_cloud = Cell::new(PointCloud::from(&args.e57_path));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shader/render_point_cs.wgsl"
            ))),
        });

        PointCloudRenderer { point_cloud }
    }

    fn process_event(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn update_render(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn resize(
        &mut self,
        _config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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

        queue.submit(Some(encoder.finish()));
    }
}
