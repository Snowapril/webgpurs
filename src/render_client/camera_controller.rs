use crate::render_client::camera::Camera;
use crate::utils::math_util::axis_angle_to_quat;
use std::{cell::RefCell, rc::Rc};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::Key,
};

#[derive(Default)]
pub struct CameraController {
    speed: f32,
    move_dir: glam::Vec2,
    cursor_delta: glam::Vec2,
    rotate_activated: bool,
    last_cursor_pos: PhysicalPosition<f64>,
    camera: Rc<RefCell<Camera>>,
}

impl CameraController {
    pub fn new(speed: f32, camera: Rc<RefCell<Camera>>) -> Self {
        Self {
            speed,
            camera: camera,
            ..Default::default()
        }
    }

    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Character(s),
                        state,
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Pressed {
                    let dir = match s.as_str() {
                        "w" => glam::Vec2::new(0.0, -1.0),
                        "a" => glam::Vec2::new(-1.0, 0.0),
                        "s" => glam::Vec2::new(0.0, 1.0),
                        "d" => glam::Vec2::new(1.0, 0.0),
                        _ => glam::Vec2::ZERO,
                    };

                    if dir.ne(&glam::Vec2::ZERO) {
                        self.move_dir = dir;
                        true
                    } else {
                        false
                    }
                } else {
                    self.move_dir = glam::Vec2::ZERO;
                    false
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Right,
                ..
            } => {
                self.rotate_activated = *state == ElementState::Pressed;
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.last_cursor_pos == PhysicalPosition::<f64>::default() {
                    self.last_cursor_pos = position.clone();
                }

                const SENSITIVITY: f32 = 8e-3;

                self.cursor_delta = glam::Vec2::new(
                    (self.last_cursor_pos.x - position.x) as f32,
                    (self.last_cursor_pos.y - position.y) as f32,
                ) * SENSITIVITY;
                self.last_cursor_pos = position.clone();

                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, _delta_time: f64) {
        let mut camera: std::cell::RefMut<'_, Camera> = self.camera.borrow_mut();

        if self.rotate_activated {
            let yaw_quat = axis_angle_to_quat(self.cursor_delta.x, camera.up);
            let pitch_quat =
                axis_angle_to_quat(self.cursor_delta.y, camera.dir.normalize().cross(camera.up));
            camera.dir = (yaw_quat * pitch_quat).normalize() * camera.dir;
        }

        camera.eye = camera.eye
            + (self.move_dir.y * camera.dir + self.move_dir.x * camera.dir.cross(camera.up))
                * self.speed;
    }
}
