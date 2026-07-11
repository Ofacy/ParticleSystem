use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Add;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::Index;
use std::ops::SubAssign;

#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vec4 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub w: f32,
}

impl Default for Vec4 {
	fn default() -> Self {
		Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
	}
}

pub type Vec3 = Vec4;

impl Vec3 {
	pub fn new3(data: [f32; 3]) -> Self {
		Vec4 { x: data[0], y: data[1], z: data[2], w: 0.0 }
	}

	pub fn length(self) -> f32 {
		(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
	}

	pub fn normalize(self) -> Self {
		let len = self.length();
		Self::new3([
			self.x / len,
			self.y / len,
			self.z / len,
		])
	}

	pub fn cross(self, rhs: Self) -> Self {
		Self::new3([
			self.y * rhs.z - self.z * rhs.y,
			self.z * rhs.x - self.x * rhs.z,
			self.x * rhs.y - self.y * rhs.x
		])
	}

	pub fn argmax(self) -> usize {
		if self.x >= self.y && self.x >= self.z {
			0
		} else if self.y >= self.x && self.y >= self.z {
			1
		} else {
			2
		}
	}

	pub fn dot(self, rhs: Self) -> f32 {
		self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
	}
}

impl Vec4 {
	pub fn new4(data: [f32; 4]) -> Self {
		Vec4 { x: data[0], y: data[1], z: data[2], w: data[3] }
	}
}

impl Mul<f32> for Vec4 {
	type Output = Self;

	fn mul(self, rhs: f32) -> Self {
		Self::new4([
			self.x * rhs,
			self.y * rhs,
			self.z * rhs,
			self.w * rhs,
		])
	}
}

impl Mul<Vec4> for f32 {
	type Output = Vec4;

	fn mul(self, rhs: Vec4) -> Vec4 {
		Vec4 {
			x: rhs.x * self,
			y: rhs.y * self,
			z: rhs.z * self,
			w: rhs.w * self,
		}
	}
}

impl Mul for Vec4 {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self {
		Self::new4([
			self.x * rhs.x,
			self.y * rhs.y,
			self.z * rhs.z,
			self.w * rhs.w,
		])
	}
}

impl Add for Vec4 {
	type Output = Self;


	fn add(self, rhs: Self) -> Self {
		Self::new4([
			self.x + rhs.x,
			self.y + rhs.y,
			self.z + rhs.z,
			self.w + rhs.w,
		])
	}
}

impl Sub for Vec4 {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		Self::new4([
			self.x - rhs.x,
			self.y - rhs.y,
			self.z - rhs.z,
			self.w - rhs.w,
		])
	}
}

impl SubAssign for Vec4 {
	fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
		self.z -= rhs.z;
		self.w -= rhs.w;
	}
}

impl AddAssign for Vec4 {
	fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
		self.z += rhs.z;
		self.w += rhs.w;
	}
}

impl Index<usize> for Vec4 {
	type Output = f32;

	fn index(&self, index: usize) -> &f32 {
		match index {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			3 => &self.w,
			_ => panic!("Index out of bounds for Vec4"),
		}
	}
}

impl Neg for Vec4 {
	type Output = Self;

	fn neg(self) -> Self {
		Self::new4([
			-self.x,
			-self.y,
			-self.z,
			-self.w,
		])
	}
}

