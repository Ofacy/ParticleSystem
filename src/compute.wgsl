
struct ParticleLifetime {
    @location(0) velocity: vec3<f32>,
    @location(1) lifetime: f32,
}
struct ParticleVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct SimulationUniforms {
    gravity_position: vec3<f32>,
    delta_time: f32,
    gravity_strength: f32,
}

@group(0) @binding(0)
var<storage, read_write> particle_lifetimes: array<ParticleLifetime>;

@group(0) @binding(1)
var<storage, read_write> particle_vertices: array<ParticleVertex>;

@group(1) @binding(0)
var<uniform> simulation_uniforms: SimulationUniforms;

@compute
@workgroup_size(64, 1, 1)
fn update_particle(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let workgroup_size = vec3<u32>(64, 1, 1);
    let particle_index = global_invocation_id.x + (global_invocation_id.y * workgroup_size.x * num_workgroups.x) +( global_invocation_id.z * workgroup_size.x * workgroup_size.y * num_workgroups.x * num_workgroups.y);
    let gravity_direction = simulation_uniforms.gravity_position - particle_vertices[particle_index].position;
    let gravity_distance = length(gravity_direction);
    particle_lifetimes[particle_index].lifetime -= simulation_uniforms.delta_time;
    particle_lifetimes[particle_index].velocity += gravity_direction * simulation_uniforms.gravity_strength * simulation_uniforms.delta_time / (gravity_distance * gravity_distance);
    particle_vertices[particle_index].position += particle_lifetimes[particle_index].velocity * simulation_uniforms.delta_time;
    particle_vertices[particle_index].color = vec3<f32>(0.2, 0.8, 0.7) * 2.0 / gravity_distance;
}

