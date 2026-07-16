
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationParameters {
	pub gravity_position: [f32; 4],
	pub starting_position: [f32; 3],
	pub starting_position_radius: f32,
	pub delta_time: f32,
	pub gravity_strength: f32,
	pub starting_lifetime: f32,
	pub _padding: [f32; 2],
}