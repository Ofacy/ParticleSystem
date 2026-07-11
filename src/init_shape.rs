#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitShapeUniforms {
	pub current_particle_offset: u32,
	pub spawn_density: u32,
	pub size: f32,
}