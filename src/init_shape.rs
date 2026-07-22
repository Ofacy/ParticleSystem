use egui::{Grid, Id, Modal};
use wgpu::{ComputePassDescriptor, include_wgsl};

use crate::particle_chunk::{self, ParticleChunk};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitShapeUniforms {
	pub starting_lifetime: [f32; 2],
	pub current_particle_offset: u32,
	pub spawn_density: u32,
	pub size: f32,
	pub _padding: [f32; 1],
}

pub enum InitShapeType {
	Cube,
	Sphere,
}

#[derive(Debug, Copy, Clone)]
pub enum InitShapeDescriptor {
	Cube {
		starting_lifetime: [f32; 2],
		spawn_density: u32,
		size: f32,
	},
	Sphere {
		starting_lifetime: [f32; 2],
		size: f32,
	},
}

#[derive(Debug, Copy, Clone)]
struct InitShapeModalUI {
	descriptor: InitShapeDescriptor,
	use_lifetimes: bool,
}

enum InitShapeModalEndOperation {
	InitShape,
	Cancel,
	None
}
pub struct InitShape {
	uniform_buffer: wgpu::Buffer,
	bind_group: wgpu::BindGroup,
	cube_pipeline: wgpu::ComputePipeline,
	sphere_pipeline: wgpu::ComputePipeline,
	modal_ui: Option<InitShapeModalUI>,
}

impl InitShape {
	pub fn new(
		device: &wgpu::Device,
		chunk_data_bind_group_layout: &wgpu::BindGroupLayout,
	) -> Self{
		let init_cube_shader = device.create_shader_module(include_wgsl!("init_cube.wgsl"));

		let init_sphere_shader = device.create_shader_module(include_wgsl!("init_sphere.wgsl"));

		let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Init Shape Uniform Buffer"),
			size: std::mem::size_of::<InitShapeUniforms>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Init Shape Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None
				}
			]
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Init Shape Bind Group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: uniform_buffer.as_entire_binding(),
				}
			]
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Init Pipeline Layout"),
			bind_group_layouts: &[Some(chunk_data_bind_group_layout), Some(&bind_group_layout)],
			immediate_size: 0,
		});

		let cube_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Init Cube Pipeline"),
			layout: Some(&pipeline_layout),
			module: &init_cube_shader,
			entry_point: Some("init_cube"),
			cache: None,
			compilation_options: Default::default(),
		});

		let sphere_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Init Sphere Pipeline"),
			layout: Some(&pipeline_layout),
			module: &init_sphere_shader,
			entry_point: Some("init_sphere"),
			cache: None,
			compilation_options: Default::default(),
		});

		Self {
			uniform_buffer,
			bind_group,
			cube_pipeline,
			sphere_pipeline,
			modal_ui: None,
		}

	}

	pub fn init_shape(
		&mut self,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		chunks: &[ParticleChunk],
		particle_count: u32,
		descriptor: &InitShapeDescriptor,
	) {
		let mut uniforms: InitShapeUniforms =
		match descriptor {
			InitShapeDescriptor::Cube { starting_lifetime, spawn_density, size } => {
				InitShapeUniforms {
					starting_lifetime: *starting_lifetime,
					current_particle_offset: 0,
					spawn_density: *spawn_density,
					size: *size,
					_padding: [0.0; 1],
				}
			}
			InitShapeDescriptor::Sphere { starting_lifetime, size } => {
				InitShapeUniforms {
					current_particle_offset: 0,
					starting_lifetime: *starting_lifetime,
					spawn_density: particle_count,
					size: *size,
					_padding: [0.0; 1],
				}
			}
		};

		let pipeline = match descriptor {
			InitShapeDescriptor::Cube { .. } => &self.cube_pipeline,
			InitShapeDescriptor::Sphere { .. } => &self.sphere_pipeline,
		};

		for particle_chunk in chunks {
			let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Init Shape Encoder"),
			});
			{
				let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
					label: Some("Init Shape Compute Pass"),
					timestamp_writes: None,
				});

				queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
				
				compute_pass.set_pipeline(pipeline);
				compute_pass.set_bind_group(1, &self.bind_group, &[]);
				particle_chunk.dispatch_update(&mut compute_pass);
				uniforms.current_particle_offset += particle_chunk.get_particle_count();
			}
			queue.submit(Some(encoder.finish()));
		}
	}

	pub fn open_modal_ui(
		&mut self,
		descriptor: InitShapeType,
	) {
		self.modal_ui = Some(InitShapeModalUI {
			use_lifetimes: false,
			descriptor: match descriptor {
				InitShapeType::Cube => InitShapeDescriptor::Cube {
					starting_lifetime: [1.0, 5.0],
					spawn_density: 13000,
					size: 0.02,
				},
				InitShapeType::Sphere => InitShapeDescriptor::Sphere {
					starting_lifetime: [1.0, 5.0],
					size: 1.0,
				},
			},
		});
	}

	pub fn modal_ui(
		&mut self,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		chunks: &[ParticleChunk],
		particle_count: u32,
		ui: &mut egui::Ui,
	) {
		let mut end_operation = InitShapeModalEndOperation::None;
		{
			let modal_ui = match &mut self.modal_ui {
				Some(modal_ui) => modal_ui,
				None => return,
			};
			Modal::new(Id::new("Init Shape Modal"))
			.show(ui.ctx(), |ui| {
				ui.label(match modal_ui.descriptor {
					InitShapeDescriptor::Cube { .. } => "Initialize Particles as Cube",
					InitShapeDescriptor::Sphere { .. } => "Initialize Particles as Sphere",
				});
				ui.separator();
				ui.label("Starting Lifetime Range");
				ui.checkbox(&mut modal_ui.use_lifetimes, "Use Starting Lifetime Range");
				if modal_ui.use_lifetimes {
					Grid::new("Init Shape Grid Lifetime Range")
						.num_columns(2)
						.show(ui, |ui| {
							ui.label("Min");
							ui.add(egui::DragValue::new(match &mut modal_ui.descriptor {
								InitShapeDescriptor::Cube { starting_lifetime, .. } => &mut starting_lifetime[0],
								InitShapeDescriptor::Sphere { starting_lifetime, .. } => &mut starting_lifetime[0],
							}).speed(0.1).range(0.0..=f32::MAX));
							ui.end_row();
							ui.label("Max");
							ui.add(egui::DragValue::new(match &mut modal_ui.descriptor {
								InitShapeDescriptor::Cube { starting_lifetime, .. } => &mut starting_lifetime[1],
								InitShapeDescriptor::Sphere { starting_lifetime, .. } => &mut starting_lifetime[1],
							}).speed(0.1).range(0.0..=f32::MAX));
							ui.end_row();
					});
				}
				ui.horizontal(|ui| {
					ui.label(match modal_ui.descriptor {
						InitShapeDescriptor::Cube { .. } => "Size Between Particles",
						InitShapeDescriptor::Sphere { .. } => "Size (Sphere Radius)",
					});
					ui.add(egui::DragValue::new(match &mut modal_ui.descriptor {
						InitShapeDescriptor::Cube { size, .. } => size,
						InitShapeDescriptor::Sphere { size, .. } => size,
					}).speed(0.001).range(0.0..=f32::MAX));
				});

				if let InitShapeDescriptor::Cube { spawn_density, .. } = &mut modal_ui.descriptor {
					ui.horizontal(|ui| {
						ui.label("Spawn Density");
						ui.add(egui::DragValue::new(spawn_density).speed(1).range(1..=u32::MAX));
					});
				}
				
				ui.horizontal(|ui| {
					if ui.button("Confirm").clicked() {
						end_operation = InitShapeModalEndOperation::InitShape;
					}
					if ui.button("Cancel").clicked() {
						end_operation = InitShapeModalEndOperation::Cancel;
					}
				});
			});
	}

		match end_operation {
			InitShapeModalEndOperation::InitShape => {
				let mut modal_ui = self.modal_ui.unwrap();
				if modal_ui.use_lifetimes == false {
					match &mut modal_ui.descriptor {
						InitShapeDescriptor::Cube { starting_lifetime, .. } => {
							*starting_lifetime = [f32::MAX, f32::MAX];
						}
						InitShapeDescriptor::Sphere { starting_lifetime, .. } => {
							*starting_lifetime = [f32::MAX, f32::MAX];
						}
					}
				}
				self.init_shape(device, queue, chunks, particle_count, &modal_ui.descriptor);
				self.modal_ui = None;
			}
			InitShapeModalEndOperation::Cancel => {
				self.modal_ui = None;
			}
			InitShapeModalEndOperation::None => {}
		}
	}
}