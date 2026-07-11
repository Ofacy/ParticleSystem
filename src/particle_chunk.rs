use crate::{particle_lifetime::ParticleLifetime, particle_vertex::ParticleVertex};

pub struct ParticleChunk {
	bind_group: wgpu::BindGroup,
	lifetime_buffer: wgpu::Buffer,
	vertex_buffer: wgpu::Buffer,
	particle_count: u32,
}

impl ParticleChunk {
	pub fn new(
		device: &wgpu::Device,
		update_bind_group_layout: &wgpu::BindGroupLayout,
		particle_count: u32,
	) -> Self {
		let lifetime_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Lifetime Buffer"),
			size: (particle_count as u64) * std::mem::size_of::<ParticleLifetime>() as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Vertex Buffer"),
			size: (particle_count as u64) * std::mem::size_of::<ParticleVertex>() as u64,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
				},
			],
		});

		Self {
			bind_group,
			lifetime_buffer,
			vertex_buffer,
			particle_count,
		}
	}

	pub fn get_lifetime_buffer(&self) -> &wgpu::Buffer {
		&self.lifetime_buffer
	}
	
	pub fn get_vertex_buffer(&self) -> &wgpu::Buffer {
		&self.vertex_buffer
	}

	pub fn get_particle_count(&self) -> u32 {
		self.particle_count
	}

	pub fn dispatch_update(
		&self,
		pass: &mut wgpu::ComputePass<'_>,
	) {
		pass.set_bind_group(0, &self.bind_group, &[]);
		pass.dispatch_workgroups(self.particle_count, 1, 1);
	}
}