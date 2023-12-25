use crate::{
    pass::{black_board, render_context},
    render_device,
};
use bytemuck::{Pod, Zeroable};
use clap::Parser;
use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
    f32::consts,
    mem,
    num::NonZeroU32,
    rc::Rc,
};
use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

pub trait RenderPass: 'static {
    fn on_resized(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device_context: &RefCell<render_device::RenderDeviceContext>,
    );

    fn process_event(&mut self, event: WindowEvent);

    fn update_render(
        &mut self,
        device_context: &RefCell<render_device::RenderDeviceContext>,
        black_board: &RefMut<black_board::BlackBoard>,
    );

    fn render(
        &mut self,
        back_buffer_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        device_context: &RefCell<render_device::RenderDeviceContext>,
        render_context: &Ref<render_context::RenderContext>,
        black_board: &RefMut<black_board::BlackBoard>,
    );
}
