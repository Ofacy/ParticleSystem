use wgpu::util::DeviceExt;

use crate::{particle_lifetime::ParticleLifetime, particle_vertex::ParticleVertex};


pub struct ParticleChunk {
	bind_group: wgpu::BindGroup,
	lifetime_buffer: wgpu::Buffer,
	vertex_buffer: wgpu::Buffer,
	particle_count_total: u32,
	particle_count_x: u32,
	particle_count_y: u32,
	particle_count_z: u32,
}

impl ParticleChunk {

	pub fn new(
		device: &wgpu::Device,
		update_bind_group_layout: &wgpu::BindGroupLayout,
		og_particle_count_goal: u32,
	) -> Self {
		let mut particle_count_goal = og_particle_count_goal;
		let dimension_limit = device.limits().max_compute_workgroups_per_dimension;

		let mut particle_count_total;
		let mut particle_count_x;
		let mut particle_count_y;
		let mut particle_count_z;
		loop {
			// particle_x must be divisible by 64 for the compute shader to work properly, so we round down to the nearest multiple of 64
			particle_count_x = std::cmp::min(particle_count_goal, dimension_limit) / 64 * 64;
			particle_count_y = std::cmp::min(
				std::cmp::max(particle_count_goal / particle_count_x, 1),
				dimension_limit,
			);
			particle_count_z = std::cmp::min(
				std::cmp::max(particle_count_goal / (particle_count_x * particle_count_y), 1),
				dimension_limit,
			);
	
			particle_count_total = particle_count_x * particle_count_y * particle_count_z;
			let memory_size = particle_count_total * std::mem::size_of::<ParticleLifetime>() as u32;
			if memory_size <= device.limits().max_storage_buffer_binding_size as u32 {
				break;
			}
			particle_count_goal = std::cmp::min(
				particle_count_goal,
				device.limits().max_storage_buffer_binding_size as u32 / std::mem::size_of::<ParticleLifetime>() as u32,
			);
		}
		println!("Creating particle chunk with {} particles ({} x {} x {})", particle_count_total, particle_count_x, particle_count_y, particle_count_z);
		assert!(particle_count_total <= particle_count_goal, "Particle count total exceeds goal");
		assert!(particle_count_x % 64 == 0, "Particle count x must be divisible by 64");
		assert!(particle_count_x <= dimension_limit, "Particle count x exceeds dimension limit");
		assert!(particle_count_y <= dimension_limit, "Particle count y exceeds dimension limit");
		assert!(particle_count_z <= dimension_limit, "Particle count z exceeds dimension limit");
		assert!(particle_count_total <= device.limits().max_storage_buffer_binding_size as u32 / std::mem::size_of::<ParticleVertex>() as u32, "Particle count total exceeds max storage buffer binding size");

		let lifetime_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Lifetime Buffer"),
			size: (particle_count_total as u64) * std::mem::size_of::<ParticleLifetime>() as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Vertex Buffer"),
			size: (particle_count_total as u64) * std::mem::size_of::<ParticleVertex>() as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false,
		});


		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Particle Bind Group"),
			layout: update_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: lifetime_buffer.as_entire_binding()
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: vertex_buffer.as_entire_binding(),
				}
			],
		});

		Self {
			bind_group,
			lifetime_buffer,
			vertex_buffer,
			particle_count_total,
			particle_count_x,
			particle_count_y,
			particle_count_z,
		}
	}

	pub fn get_lifetime_buffer(&self) -> &wgpu::Buffer {
		&self.lifetime_buffer
	}
	
	pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
		&self.vertex_buffer
	}

	pub fn get_particle_count(&self) -> u32 {
		self.particle_count_total
	}

	pub fn dispatch_update(
		&self,
		pass: &mut wgpu::ComputePass<'_>,
	) {
		pass.set_bind_group(0, &self.bind_group, &[]);
		pass.dispatch_workgroups(self.particle_count_x / 64, self.particle_count_y, self.particle_count_z);
	}
}