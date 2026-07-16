use crate::particle_chunk::ParticleChunk;


pub trait Renderer {
	fn render(
		&self,
		chunk: &ParticleChunk,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
		depth_texture_view: &wgpu::TextureView,
		simulation_params_bind_group: &wgpu::BindGroup,
		view_proj_bind_group: &wgpu::BindGroup,
		current_load_ops: (wgpu::LoadOp<wgpu::Color>, wgpu::LoadOp<f32>),
	);

	fn get_simulation_bounds(&self) -> Option<(f32, f32, f32, f32)> {
		None
	}
}
