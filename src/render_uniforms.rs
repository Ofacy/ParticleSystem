use crate::matrix4::Matrix4;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderUniforms {
	pub projection: Matrix4,
	pub view: Matrix4,
}