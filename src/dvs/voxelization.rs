use crate::pass::{render_context, render_pass};
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{borrow::Cow, cell::Cell, cell::RefCell, f32::consts, mem, num::NonZeroU32, rc::Rc};
use wgpu::util::DeviceExt;

pub struct VoxelizationPass {}

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
