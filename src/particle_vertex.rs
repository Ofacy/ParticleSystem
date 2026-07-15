#[repr(C)]
pub struct ParticleVertex {
    position: [f32; 4],
}

impl ParticleVertex {
	const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x4];

	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<ParticleVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}