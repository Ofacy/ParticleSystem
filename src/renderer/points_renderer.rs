use wgpu::util::DeviceExt;

use crate::{particle_chunk::ParticleChunk, particle_vertex::ParticleVertex, renderer::renderer::Renderer, texture::Texture};

pub struct PointsRenderer {
	pipeline: wgpu::RenderPipeline,
	uniform_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup,
}
 
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointsRendererUniforms {
	color: [f32; 4],
}

impl PointsRenderer {
	pub fn new(
		device: &wgpu::Device,
		simulation_params_group_layout: &wgpu::BindGroupLayout,
		view_proj_group_layout: &wgpu::BindGroupLayout,
		config: &wgpu::SurfaceConfiguration,
	) -> Self {
		let shader_module = device.create_shader_module(wgpu::include_wgsl!("points_renderer.wgsl"));

		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Points Renderer Uniform Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			contents: bytemuck::cast_slice(&[PointsRendererUniforms {
				color: [0.3, 0.7, 0.9, 0.2], // Example color (RGBA)
			}]),
		});

		let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Points Renderer Uniform Bind Group Layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None
				},
				count: None
			}],
		});

		let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Points Renderer Uniform Bind Group"),
			layout: &uniform_bind_group_layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: uniform_buffer.as_entire_binding(),
			}],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Points Renderer Pipeline Layout"),
			bind_group_layouts: &[Some(view_proj_group_layout), Some(simulation_params_group_layout), Some(&uniform_bind_group_layout)],
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
				fragment: Some(wgpu::FragmentState {
					module: &shader_module,
					entry_point: Some("fs_main"),
					targets: &[Some(wgpu::ColorTargetState {
						format: config.format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})],
					compilation_options: wgpu::PipelineCompilationOptions::default(),
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::PointList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: None,
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
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
			}),
			uniform_buffer,
			uniform_bind_group,
		}
	}

	pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: PointsRendererUniforms) {;
		queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
	}
}

impl Renderer for  PointsRenderer {
	fn render_chunk(
		&self,
		chunk: &ParticleChunk,
		_chunk_index: usize,
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
		render_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
		render_pass.set_vertex_buffer(0, chunk.get_vertex_buffer().slice(..));
		render_pass.draw(0..chunk.get_particle_count(), 0..1);

	}
 }