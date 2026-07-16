// Vertex shader

struct ViewProjectionUniforms {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct ParticleVertex {
    @location(0) position: vec3<f32>,
}

struct SimulationUniforms {
    gravity_position: vec3<f32>,
    starting_position: vec3<f32>,
    starting_position_radius: f32,
    delta_time: f32,
    gravity_strength: f32,
    starting_lifetime: f32,
}

@group(0) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

@group(1) @binding(0)
var<uniform> simulation_uniforms: SimulationUniforms;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    in: ParticleVertex
) -> VertexOutput {
    var out: VertexOutput;

    out.color = vec3<f32>(0.2, 0.8, 0.7) * 2.0 / length(simulation_uniforms.gravity_position - in.position);
    out.clip_position = view_proj_uniforms.projection * view_proj_uniforms.view * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}


