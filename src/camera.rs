use crate::{matrix4::Matrix4, quaternion::Quaternion, render_uniforms::RenderUniforms, vector::{Vec3, Vec4}};
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

impl Camera {
	pub fn new(position: Vec3, rotation: Quaternion, fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
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

	pub fn get_render_uniforms(&self) -> RenderUniforms {
		RenderUniforms {
			view: self.get_view_matrix(),
			projection: self.projection_matrix,
		}
	}

	pub fn get_position(&self) -> Vec3 {
		self.position
	}
	
	pub fn get_rotation(&self) -> Quaternion {
		self.rotation
	}

	pub fn get_fov(&self) -> f32 {
		self.fov
	}

	pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
		self.projection_matrix = Matrix4::perspective(self.fov, aspect_ratio, self.near, self.far);
	}

	pub fn get_view_matrix(&self) -> Matrix4 {
		Matrix4::translation(-self.position) * self.rotation.inverse().to_matrix4()
	}

	pub fn get_direction_from_screen_coordinates(&self, x: f32, y: f32, width: f32, height: f32) -> Option<Vec3> {
		let aspect_ratio = width / height;
		let fov_rad = self.fov;
		let tan_fov = (fov_rad / 2.0).tan();

		// Convert screen coordinates to normalized device coordinates (NDC)
		let ndc_x = (2.0 * x / width) - 1.0;
		let ndc_y = 1.0 - (2.0 * y / height);

		let dir_camera_space = Vec3::new3([
			ndc_x * aspect_ratio * tan_fov,
			ndc_y * tan_fov,
			-1.0,
		]);

		let direction_world_space = self.rotation.to_matrix4().transform_direction(dir_camera_space).normalize();
		Some(direction_world_space)

	}

	pub fn update(&mut self, delta_time: f32) {
		if (self.key_states & 0b10000) == 0 {
			return;
		}
		let speed = 2.0;
		let forward = self.rotation.to_direction();
		let right = Vec3::new3([forward.z, 0.0, -forward.x]).normalize();

		if self.key_states & 0b00001 != 0 {
			self.position -= forward * speed * delta_time;
		}
		if self.key_states & 0b00010 != 0 {
			self.position += forward * speed * delta_time;
		}
		if self.key_states & 0b00100 != 0 {
			self.position -= right * speed * delta_time;
		}
		if self.key_states & 0b01000 != 0 {
			self.position += right * speed * delta_time;
		}
	}

	pub fn handle_input(&mut self, code: winit::keyboard::KeyCode, is_pressed: bool) {
		match code {
			KeyCode::KeyW => self.key_states = if is_pressed { self.key_states | 0b0001 } else { self.key_states & !0b0001 },
			KeyCode::KeyS => self.key_states = if is_pressed { self.key_states | 0b0010 } else { self.key_states & !0b0010 },
			KeyCode::KeyA => self.key_states = if is_pressed { self.key_states | 0b0100 } else { self.key_states & !0b0100 },
			KeyCode::KeyD => self.key_states = if is_pressed { self.key_states | 0b1000 } else { self.key_states & !0b1000 },
			_ => {}
		}
	}

	pub fn handle_mouse_movement(&mut self, delta_x: f32, delta_y: f32) {
		if (self.key_states & 0b10000) == 0 {
			return;
		}
		let sensitivity = 0.002;
		let yaw = delta_x * sensitivity;
		let pitch = delta_y * sensitivity;

		let yaw_quat = Quaternion::rotate(Vec3::new3([0.0, 1.0, 0.0]), -yaw);
		let pitch_quat = Quaternion::rotate(Vec3::new3([1.0, 0.0, 0.0]), -pitch);

		self.rotation = yaw_quat * self.rotation;
		self.rotation = self.rotation * pitch_quat;
	}

	pub fn handle_mouse_button(&mut self, button: winit::event::MouseButton, is_pressed: bool) {
		// Implement mouse button handling if needed
		match button {
			winit::event::MouseButton::Right => {
				self.key_states = if is_pressed { self.key_states | 0b10000 } else { self.key_states & !0b10000 };
			}
			_ => {}
		}
	}
}