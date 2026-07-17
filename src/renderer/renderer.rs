use crate::particle_chunk::ParticleChunk;

pub trait Renderer {
	// called after the simulation step to render the particle chunk (can be used to compute additional data for the next frame)
	fn render_chunk(
		&self,
		_chunk: &ParticleChunk,
		_chunk_index: usize,
		_encoder: &mut wgpu::CommandEncoder,
		_view: &wgpu::TextureView,
		_depth_texture_view: &wgpu::TextureView,
		_simulation_params_bind_group: &wgpu::BindGroup,
		_view_proj_bind_group: &wgpu::BindGroup,
		_current_load_ops: (wgpu::LoadOp<wgpu::Color>, wgpu::LoadOp<f32>),
	) {}

	fn render_frame(
		&self,
		_encoder: &mut wgpu::CommandEncoder,
		_view: &wgpu::TextureView,
		_depth_texture_view: &wgpu::TextureView,
		_simulation_params_bind_group: &wgpu::BindGroup,
		_view_proj_bind_group: &wgpu::BindGroup,
		_current_load_ops: (wgpu::LoadOp<wgpu::Color>, wgpu::LoadOp<f32>),
	) {}

	fn get_simulation_bounds(&self) -> Option<(f32, f32, f32, f32)> {
		None
	}
}
