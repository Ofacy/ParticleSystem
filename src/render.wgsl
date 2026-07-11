// Vertex shader

struct RenderUniforms {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct ParticleVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

@group(0) @binding(0)
var<uniform> render_uniforms: RenderUniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in: ParticleVertex
) -> VertexOutput {
    var out: VertexOutput;

    out.color = in.color;
    out.clip_position = render_uniforms.projection * render_uniforms.view * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}


