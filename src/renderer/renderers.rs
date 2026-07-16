use crate::renderer::points_renderer::PointsRenderer;

pub struct Renderers {
	points_renderer: PointsRenderer,
}

impl Renderers {
	pub fn new(
		device: &wgpu::Device,
		simulation_params_group_layout: &wgpu::BindGroupLayout,
		view_proj_group_layout: &wgpu::BindGroupLayout,
		config: &wgpu::SurfaceConfiguration,
	) -> Self {
		Self {
			points_renderer: PointsRenderer::new(device, simulation_params_group_layout, view_proj_group_layout, config),
		}
	}

	pub fn get_points_renderer(&self) -> &PointsRenderer {
		&self.points_renderer
	}
}
