use crate::{particle_chunk::ParticleChunk, renderer::{points_renderer::PointsRenderer, renderer::Renderer}};

pub struct Renderers {
	points_renderer: PointsRenderer,
}

pub enum RendererType {
	Points,
}

impl Renderers {
	pub fn new(
		device: &wgpu::Device,
		_particle_chunks: &[ParticleChunk],
		simulation_params_group_layout: &wgpu::BindGroupLayout,
		view_proj_group_layout: &wgpu::BindGroupLayout,
		config: &wgpu::SurfaceConfiguration,
	) -> Self {
		Self {
			points_renderer: PointsRenderer::new(device, simulation_params_group_layout, view_proj_group_layout, config),
		}
	}

	pub fn get_renderer(
		&self,
		renderer_type: RendererType,
	) -> &dyn Renderer {
		match renderer_type {
			RendererType::Points => &self.points_renderer,
		}
	}
}
