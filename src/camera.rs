use crate::{
    matrix4::Matrix4, quaternion::Quaternion, vector::Vec3,
    view_proj_uniforms::ViewProjectionUniforms,
};
use winit::keyboard::KeyCode;

pub struct Camera {
    position: Vec3,
    rotation: Quaternion,
    projection_matrix: Matrix4,
    fov: f32,
    near: f32,
    far: f32,
    key_states: u8,
}

const KEY_FORWARD: u8 = 0b0000001;
const KEY_BACKWARD: u8 = 0b0000010;
const KEY_LEFT: u8 = 0b0000100;
const KEY_RIGHT: u8 = 0b0001000;
const KEY_ENABLE: u8 = 0b0010000;
const KEY_ROLL_LEFT: u8 = 0b0100000;
const KEY_ROLL_RIGHT: u8 = 0b1000000;

impl Camera {
    pub fn new(
        position: Vec3,
        rotation: Quaternion,
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            rotation,
            projection_matrix: Matrix4::perspective(fov, aspect_ratio, near, far),
            fov,
            near,
            far,
            key_states: 0,
        }
    }

    pub fn get_render_uniforms(&self) -> ViewProjectionUniforms {
        ViewProjectionUniforms {
            view: self.get_view_matrix(),
            projection: self.projection_matrix,
        }
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.projection_matrix = Matrix4::perspective(self.fov, aspect_ratio, self.near, self.far);
    }

    pub fn get_view_matrix(&self) -> Matrix4 {
        Matrix4::translation(-self.position) * self.rotation.inverse().to_matrix4()
    }

    pub fn get_direction_from_screen_coordinates(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Vec3 {
        let aspect_ratio = width / height;
        let fov_rad = self.fov;
        let tan_fov = (fov_rad / 2.0).tan();

        // Convert screen coordinates to normalized device coordinates (NDC)
        let ndc_x = (2.0 * x / width) - 1.0;
        let ndc_y = 1.0 - (2.0 * y / height);

        let dir_camera_space = Vec3::new3([ndc_x * aspect_ratio * tan_fov, ndc_y * tan_fov, -1.0]);

        self.rotation
            .to_matrix4()
            .transform_direction(dir_camera_space)
            .normalize()
    }

    pub fn update(&mut self, delta_time: f32) {
        if (self.key_states & KEY_ENABLE) == 0 {
            return;
        }
        let speed = 2.0;
        let forward = self.rotation.to_direction();
        let right = self
            .rotation
            .to_matrix4()
            .transform_direction(Vec3::new3([1.0, 0.0, 0.0]))
            .normalize();

        if self.key_states & KEY_FORWARD != 0 {
            self.position -= forward * speed * delta_time;
        }
        if self.key_states & KEY_BACKWARD != 0 {
            self.position += forward * speed * delta_time;
        }
        if self.key_states & KEY_LEFT != 0 {
            self.position -= right * speed * delta_time;
        }
        if self.key_states & KEY_RIGHT != 0 {
            self.position += right * speed * delta_time;
        }
        if self.key_states & KEY_ROLL_LEFT != 0 {
            self.rotation = Quaternion::rotate(forward, delta_time) * self.rotation;
        }
        if self.key_states & KEY_ROLL_RIGHT != 0 {
            self.rotation = Quaternion::rotate(forward, -delta_time) * self.rotation;
        }
    }

    pub fn handle_input(&mut self, code: winit::keyboard::KeyCode, is_pressed: bool) {
        match code {
            KeyCode::KeyW => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_FORWARD
                } else {
                    self.key_states & !KEY_FORWARD
                }
            }
            KeyCode::KeyS => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_BACKWARD
                } else {
                    self.key_states & !KEY_BACKWARD
                }
            }
            KeyCode::KeyA => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_LEFT
                } else {
                    self.key_states & !KEY_LEFT
                }
            }
            KeyCode::KeyD => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_RIGHT
                } else {
                    self.key_states & !KEY_RIGHT
                }
            }
            KeyCode::KeyE => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_ROLL_RIGHT
                } else {
                    self.key_states & !KEY_ROLL_RIGHT
                }
            }
            KeyCode::KeyQ => {
                self.key_states = if is_pressed {
                    self.key_states | KEY_ROLL_LEFT
                } else {
                    self.key_states & !KEY_ROLL_LEFT
                }
            }
            _ => {}
        }
    }

    pub fn handle_mouse_movement(&mut self, delta_x: f32, delta_y: f32) {
        if (self.key_states & KEY_ENABLE) == 0 {
            return;
        }
        let sensitivity = 0.002;

        let up = self
            .rotation
            .to_matrix4()
            .transform_direction(Vec3::new3([0.0, 1.0, 0.0]));
        let right = Vec3::new3([1.0, 0.0, 0.0]);

        let yaw_rotation = Quaternion::rotate(up, -delta_x * sensitivity);
        let pitch_rotation = Quaternion::rotate(right, -delta_y * sensitivity);

        self.rotation = yaw_rotation * self.rotation;
        self.rotation = self.rotation * pitch_rotation;
    }

    pub fn handle_mouse_button(&mut self, button: winit::event::MouseButton, is_pressed: bool) {
        // Implement mouse button handling if needed
        if button == winit::event::MouseButton::Right {
            self.key_states = if is_pressed {
                self.key_states | KEY_ENABLE
            } else {
                self.key_states & !KEY_ENABLE
            };
        }
    }
}
