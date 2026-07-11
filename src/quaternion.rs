use std::ops::Mul;

use crate::{matrix4::Matrix4, vector::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
	pub w: f32,
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Quaternion {
	pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
		Self { w, x, y, z }
	}

	pub fn normalize(mut self) -> Self {
		let norm = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
		if norm > 0.0 {
			self.w /= norm;
			self.x /= norm;
			self.y /= norm;
			self.z /= norm;
		}
		self
	}

	pub fn from_vec(direction: Vec3, up: Vec3) -> Self {
		let f = direction.normalize();
		let r = up.cross(f).normalize();
		let u = f.cross(r);

		let trace = r.x + u.y + f.z;
		if trace > 0.0 {
			let s = (trace + 1.0).sqrt() * 2.0;
			Self {
				w: 0.25 * s,
				x: (u.z - f.y) / s,
				y: (f.x - r.z) / s,
				z: (r.y - u.x) / s,
			}
		} else if (r.x > u.y) && (r.x > f.z) {
			let s = (1.0 + r.x - u.y - f.z).sqrt() * 2.0;
			Self {
				w: (u.z - f.y) / s,
				x: 0.25 * s,
				y: (u.x + r.y) / s,
				z: (f.x + r.z) / s,
			}
		} else if u.y > f.z {
			let s = (1.0 + u.y - r.x - f.z).sqrt() * 2.0;
			Self {
				w: (f.x - r.z) / s,
				x: (u.x + r.y) / s,
				y: 0.25 * s,
				z: (f.y + u.z) / s,
			}
		} else {
			let s = (1.0 + f.z - r.x - u.y).sqrt() * 2.0;
			Self {
				w: (r.y - u.x) / s,
				x: (f.x + r.z) / s,
				y: (f.y + u.z) / s,
				z: 0.25 * s,
			}
		}
	}
	
	pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
		let half_angle = angle / 2.0;
		let sin_half_angle = half_angle.sin();
		let cos_half_angle = half_angle.cos();

		Self {
			w: cos_half_angle,
			x: axis.x * sin_half_angle,
			y: axis.y * sin_half_angle,
			z: axis.z * sin_half_angle,
		}
	}

	pub fn inverse(self) -> Self {
		let norm_sq = self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z;
		Self {
			w: self.w / norm_sq,
			x: -self.x / norm_sq,
			y: -self.y / norm_sq,
			z: -self.z / norm_sq,
		}
	}

	pub fn identity() -> Self {
		Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
	}

	pub fn dot(self, rhs: Self) -> f32 {
		self.w * rhs.w + self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
	}

	pub fn rotate(axis: Vec3, angle: f32) -> Quaternion {
		let half_angle = angle / 2.0;
		let sin_half_angle = half_angle.sin();
		let cos_half_angle = half_angle.cos();

		Quaternion::new(
			cos_half_angle,
			axis.x * sin_half_angle,
			axis.y * sin_half_angle,
			axis.z * sin_half_angle
		)
	}

	pub fn to_direction(self) -> Vec3 {
		Vec3::new3([
			2.0 * (self.x * self.z + self.w * self.y),
			2.0 * (self.y * self.z - self.w * self.x),
			1.0 - 2.0 * (self.x * self.x + self.y * self.y)
		]).normalize()
	}

	pub fn to_matrix4(self) -> Matrix4 {
		Matrix4 {
			data: [
				1.0 - 2.0 * (self.y * self.y + self.z * self.z), 2.0 * (self.x * self.y + self.w * self.z), 2.0 * (self.x * self.z - self.w * self.y), 0.0,
				2.0 * (self.x * self.y - self.w * self.z), 1.0 - 2.0 * (self.x * self.x + self.z * self.z), 2.0 * (self.y * self.z + self.w * self.x), 0.0,
				2.0 * (self.x * self.z + self.w * self.y), 2.0 * (self.y * self.z - self.w * self.x), 1.0 - 2.0 * (self.x * self.x + self.y * self.y), 0.0,
				0.0, 0.0, 0.0, 1.0
			]
		}

	}
}

impl Mul for Quaternion {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self {
			w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
			x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
			y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
			z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w
		}
	}
}