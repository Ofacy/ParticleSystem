
struct ParticleLifetime {
    @location(0) velocity: vec3<f32>,
    @location(1) lifetime: f32,
}
struct ParticleVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

@group(0) @binding(0)
var<storage, read_write> particle_lifetimes: array<ParticleLifetime>;

@group(0) @binding(1)
var<storage, read_write> particle_vertices: array<ParticleVertex>;

@group(1) @binding(0)
var<uniform> cube_scale: f32;

@compute
@workgroup_size(64)
fn init_cube(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    particle_lifetimes[global_invocation_id.x].velocity = vec3<f32>(0.0, 0.0, 0.0);
    particle_lifetimes[global_invocation_id.x].lifetime = 0.0;
    particle_vertices[global_invocation_id.x].position = vec3<f32>(f32(global_invocation_id.x) * cube_scale, 0.0, 0.0);
    particle_vertices[global_invocation_id.x].color = vec3<f32>(1.0, 1.0, 1.0);
}

