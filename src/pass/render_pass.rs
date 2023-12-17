use crate::pass::render_context;
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{borrow::Cow, cell::RefCell, f32::consts, mem, num::NonZeroU32, rc::Rc};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

pub trait RenderPass: 'static {
    fn on_resized(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    fn process_event(&mut self, event: WindowEvent);

    fn update_render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );
}
