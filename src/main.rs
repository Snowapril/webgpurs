mod dvs;
mod pass;
mod point_cloud;
mod render_client;
mod samples;
mod scene;
mod utils;

use std::sync::Arc;

use dvs::deferred_voxel_shading;
use point_cloud::point_cloud_renderer;
use scene::obj_loader;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    keyboard::{Key, NamedKey},
    window::Window,
};

use render_client::{render_device, surface_wrapper};
use utils::{counter, logger};

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
}

impl EventLoopWrapper {
    pub fn new(title: &str) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title(title);
        let window = Arc::new(builder.build(&event_loop).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("Couldn't get canvas");
            canvas.style().set_css_text("height: 100%; width: 100%;");
            // On wasm, append the canvas to the document body
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&canvas).ok())
                .expect("couldn't append canvas to document body");
        }

        Self { event_loop, window }
    }
}

async fn start<E: render_device::RenderDevice>(title: &str) {
    logger::init_logger();
    let window_loop = EventLoopWrapper::new(title);
    let mut surface = surface_wrapper::SurfaceWrapper::new();
    let context = render_device::RenderDeviceContext::init_async::<E>(
        &mut surface,
        window_loop.window.clone(),
    )
    .await;
    let mut frame_counter = counter::FrameCounter::new();

    // We wait to create the render_device until we have a valid surface.
    let mut render_device = None;

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            let event_loop_function = EventLoop::spawn;
        } else {
            let event_loop_function = EventLoop::run;
        }
    }

    log::info!("Entering event loop...");
    // On native this is a result, but on wasm it's a unit type.
    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &EventLoopWindowTarget<()>| {
            // We set to refresh as fast as possible.
            target.set_control_flow(ControlFlow::Poll);

            match event {
                ref e if surface_wrapper::SurfaceWrapper::start_condition(e) => {
                    surface.resume(&context, window_loop.window.clone(), E::SRGB);

                    // If we haven't created the render_device yet, do so now.
                    if render_device.is_none() {
                        render_device = Some(E::init(
                            surface.config(),
                            &context.adapter,
                            &context.device,
                            &context.queue,
                        ));
                    }
                }
                Event::Suspended => {
                    surface.suspend();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        surface.resize(&context, size);
                        render_device.as_mut().unwrap().resize(
                            surface.config(),
                            &context.device,
                            &context.queue,
                        );

                        window_loop.window.request_redraw();
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Character(s),
                                ..
                            },
                        ..
                    } if s == "r" => {
                        println!("{:#?}", context.instance.generate_report());
                    }
                    WindowEvent::RedrawRequested => {
                        frame_counter.update();

                        let frame = surface.acquire(&context);
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                            format: Some(surface.config().view_formats[0]),
                            ..wgpu::TextureViewDescriptor::default()
                        });

                        render_device
                            .as_mut()
                            .unwrap()
                            .update_render(&context.device, &context.queue);

                        render_device.as_mut().unwrap().render(
                            &view,
                            &context.device,
                            &context.queue,
                        );

                        frame.present();

                        window_loop.window.request_redraw();
                    }
                    _ => render_device.as_mut().unwrap().process_event(event),
                },
                _ => {}
            }
        },
    );
}

pub fn run<E: render_device::RenderDevice>(title: &'static str) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_bindgen_futures::spawn_local(async move { start::<E>(title).await })
        } else {
            pollster::block_on(start::<E>(title));
        }
    }
}

fn main() {
    // run::<samples::cube_scene_renderer::CubeSceneRenderer>("cube");
    //run::<point_cloud_renderer::PointCloudRenderer>("PointCloudRenderer");
    scene::obj_loader::load_obj("./resources/cornell-box.obj");

    run::<deferred_voxel_shading::DeferredVoxelShading>("DeferredVoxelShading");
}
