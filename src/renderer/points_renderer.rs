use crate::{particle_chunk::ParticleChunk, particle_vertex::ParticleVertex, renderer::renderer::Renderer, texture::Texture};

pub struct PointsRenderer {
	pipeline: wgpu::RenderPipeline,
}
 
impl PointsRenderer {
	pub fn new(
		device: &wgpu::Device,
		simulation_params_group_layout: &wgpu::BindGroupLayout,
		view_proj_group_layout: &wgpu::BindGroupLayout,
		config: &wgpu::SurfaceConfiguration,
	) -> Self {
		let shader_module = device.create_shader_module(wgpu::include_wgsl!("points_renderer.wgsl"));



		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Points Renderer Pipeline Layout"),
			bind_group_layouts: &[Some(view_proj_group_layout), Some(simulation_params_group_layout)],
			immediate_size: 0,
		});

		Self {
			pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Points Renderer Pipeline"),
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader_module,
					entry_point: Some("vs_main"),
					buffers: &[ParticleVertex::desc()],
					compilation_options: wgpu::PipelineCompilationOptions::default(),
				},
				fragment: Some(wgpu::FragmentState { // 3.
					module: &shader_module,
					entry_point: Some("fs_main"),
					targets: &[Some(wgpu::ColorTargetState { // 4.
						format: config.format,
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
					compilation_options: wgpu::PipelineCompilationOptions::default(),
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::PointList, // 1.
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw, // 2.
					cull_mode: None,
					// Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
					polygon_mode: wgpu::PolygonMode::Fill,
					// Requires Features::DEPTH_CLIP_CONTROL
					unclipped_depth: false,
					// Requires Features::CONSERVATIVE_RASTERIZATION
					conservative: false,
				},
				depth_stencil: Some(wgpu::DepthStencilState {
					format: Texture::DEPTH_FORMAT,
					depth_write_enabled: Some(true),
					depth_compare: Some(wgpu::CompareFunction::Less),
					stencil: wgpu::StencilState::default(),
					bias: wgpu::DepthBiasState::default(),
				}),
					multisample: wgpu::MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false,
				},
				multiview_mask: None,
				cache: None,
			})
		}
	}
}

impl Renderer for  PointsRenderer {
	fn render(
		&self,
		chunk: &ParticleChunk,
		encoder: &mut wgpu::CommandEncoder,
		view: &wgpu::TextureView,
		depth_texture_view: &wgpu::TextureView,
		simulation_params_bind_group: &wgpu::BindGroup,
		view_proj_bind_group: &wgpu::BindGroup,
		current_load_ops: (wgpu::LoadOp<wgpu::Color>, wgpu::LoadOp<f32>),
	) {
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Points Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view,
				depth_slice: None,
				resolve_target: None,
				ops: wgpu::Operations {
					load: current_load_ops.0,
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
				view: &depth_texture_view,
				depth_ops: Some(wgpu::Operations {
					load: current_load_ops.1,
					store: wgpu::StoreOp::Store,
				}),
				stencil_ops: None,
			}),
			occlusion_query_set: None,
			timestamp_writes: None,
			multiview_mask: None,
		});
		render_pass.set_pipeline(&self.pipeline);
		render_pass.set_bind_group(0, view_proj_bind_group, &[]);
		render_pass.set_bind_group(1, simulation_params_bind_group, &[]);
		render_pass.set_vertex_buffer(0, chunk.get_vertex_buffer().slice(..));
		render_pass.draw(0..chunk.get_particle_count(), 0..1);

	}
 }