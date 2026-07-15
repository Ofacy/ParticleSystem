#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitShapeUniforms {
	pub starting_lifetime: [f32; 2],
	pub current_particle_offset: u32,
	pub spawn_density: u32,
	pub size: f32,
	pub _padding: [f32; 1],
}