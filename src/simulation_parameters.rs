use crate::vector::Vec3;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationParameters {
	pub gravity_position: Vec3,
	pub gravity_strength: f32,
	pub _padding: f32,
	pub _padding2: f32,
	pub _padding3: f32,
}