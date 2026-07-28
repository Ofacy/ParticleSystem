use wgpu::RenderPass;

use crate::particle_chunk::ParticleChunk;

pub trait Renderer {
	// called after the simulation step to render the particle chunk (can be used to compute additional data for the next frame)
	fn render_chunk(
		&self,
		_chunk: &ParticleChunk,
		_chunk_index: usize,
		_render_pass: &mut RenderPass<'_>,
	) {}

	fn render_frame(
		&self,
		_render_pass: &mut RenderPass<'_>,
	) {}
}
