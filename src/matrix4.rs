use std::ops::Mul;

use crate::vector::{Vec3, Vec4};


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Matrix4 {
	pub data: [f32; 16],
}

impl Matrix4 {

	pub fn new(data: [f32; 16]) -> Self {
		Self { data }
	}

	pub fn identity() -> Self {
		Self {
			data: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0
			]
		}
	}

	pub fn inverse(&self) -> Self {
		Self {
			data: [
				self.data[0], self.data[4], self.data[8], 0.0,
				self.data[1], self.data[5], self.data[9], 0.0,
				self.data[2], self.data[6], self.data[10], 0.0,
				-(self.data[12] * self.data[0] + self.data[13] * self.data[1] + self.data[14] * self.data[2]),
				-(self.data[12] * self.data[4] + self.data[13] * self.data[5] + self.data[14] * self.data[6]),
				-(self.data[12] * self.data[8] + self.data[13] * self.data[9] + self.data[14] * self.data[10]),
				1.0
			]
		}
	}

	pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
		let rl = right - left;
		let tb = top - bottom;
		let fn_ = far - near;

		Self {
			data: [
				2.0 / rl, 0.0, 0.0, 0.0,
				0.0, 2.0 / tb, 0.0, 0.0,
				0.0, 0.0, -2.0 / fn_, 0.0,
				-(right + left) / rl, -(top + bottom) / tb, -(far + near) / fn_, 1.0
			]
		}
	}

	pub fn transpose(&self) -> Self {
		let mut transposed = [0.0; 16];
		for i in 0..4 {
			for j in 0..4 {
				transposed[i * 4 + j] = self.data[j * 4 + i];
			}
		}
		Self { data: transposed }
	}

	pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
		let f = 1.0 / (fov_y / 2.0).tan();
		let nf = 1.0 / (near - far);

		Self {
			data: [
				f / aspect, 0.0, 0.0, 0.0,
				0.0, f, 0.0, 0.0,
				0.0, 0.0, (far + near) * nf, -1.0,
				0.0, 0.0, -(2.0 * far * near) / (far - near), 0.0
			]
		}
	}

	pub fn view(eye: Vec3, center: Vec3, up: Vec3) -> Self {
		let f = (center - eye).normalize();
		let r = up.cross(f).normalize();
		let u = f.cross(r);

		Self {
			data: [
				r.x, u.x, -f.x, 0.0,
				r.y, u.y, -f.y, 0.0,
				r.z, u.z, -f.z, 0.0,
				-r.dot(eye), -u.dot(eye), f.dot(eye), 1.0,
			]
		}
	}

	pub fn translation(v: Vec3) -> Self {
		Self {
			data: [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				v.x, v.y, v.z, 1.0
			]
		}
	}
}

impl Mul for Matrix4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		let mut result = Self::identity();

		for i in 0..4 {
			for j in 0..4 {
				result.data[i * 4 + j] = self.data[i * 4 + 0] * rhs.data[0 * 4 + j]
					+ self.data[i * 4 + 1] * rhs.data[1 * 4 + j]
					+ self.data[i * 4 + 2] * rhs.data[2 * 4 + j]
					+ self.data[i * 4 + 3] * rhs.data[3 * 4 + j];
			}
		}
		result
	}
}

impl Mul<Vec4> for Matrix4 {
	type Output = Vec4;

	fn mul(self, rhs: Vec4) -> Self::Output {
		Vec4::new4([
			self.data[0] * rhs.x + self.data[4] * rhs.y + self.data[8] * rhs.z + self.data[12],
			self.data[1] * rhs.x + self.data[5] * rhs.y + self.data[9] * rhs.z + self.data[13],
			self.data[2] * rhs.x + self.data[6] * rhs.y + self.data[10] * rhs.z + self.data[14],
			self.data[3] * rhs.x + self.data[7] * rhs.y + self.data[11] * rhs.z + self.data[15]
		])
	}
}
