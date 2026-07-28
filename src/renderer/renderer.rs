use wgpu::RenderPass;

use crate::particle_chunk::ParticleChunk;

pub trait Renderer {
    // used to compute extra stuff
    fn compute_chunk(
        &self,
        _chunk: &ParticleChunk,
        _chunk_index: usize,
        _compute_pass: &mut wgpu::ComputePass<'_>,
    ) {
    }

    fn render_chunk(
        &self,
        _chunk: &ParticleChunk,
        _chunk_index: usize,
        _render_pass: &mut RenderPass<'_>,
    ) {
    }

    fn render_frame(&self, _render_pass: &mut RenderPass<'_>) {}
}
