
struct ParticleLifetime {
    @location(0) velocity: vec3<f32>,
    @location(1) lifetime: f32,
}
struct ParticleVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct InitSphereUniforms {
    current_particle_offset: u32,
    spawn_density: u32,
    radius: f32,
}

struct ParticleChunkUniforms {
    x_count: u32,
    y_count: u32,
}

@group(0) @binding(0)
var<storage, read_write> particle_lifetimes: array<ParticleLifetime>;

@group(0) @binding(1)
var<storage, read_write> particle_vertices: array<ParticleVertex>;

@group(0) @binding(2)
var<uniform> chunk_uniforms: ParticleChunkUniforms;

@group(1) @binding(0)
var<uniform> init_sphere_uniforms: InitSphereUniforms;

@compute
@workgroup_size(64, 1, 1)
fn init_sphere(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let workgroup_size = vec3<u32>(64, 1, 1);
    let particle_index = global_invocation_id.x;
    let real_index = particle_index + init_sphere_uniforms.current_particle_offset;
    let particle_count_per_axis = u32(sqrt(f32(init_sphere_uniforms.spawn_density)));
    let phi = 2.0 * 3.14159265358979323846 * f32(real_index % particle_count_per_axis) / f32(particle_count_per_axis);
    let theta = 3.14159265358979323846 * f32((real_index / particle_count_per_axis) % particle_count_per_axis) / f32(particle_count_per_axis);

    let position = vec3<f32>(
        init_sphere_uniforms.radius * sin(theta) * cos(phi),
        init_sphere_uniforms.radius * sin(theta) * sin(phi),
        init_sphere_uniforms.radius * cos(theta)
    );
    particle_lifetimes[particle_index].velocity = vec3<f32>(0.0, 0.0, 0.0);
    particle_lifetimes[particle_index].lifetime = 0.0;
    particle_vertices[particle_index].position = position;
    particle_vertices[particle_index].color = vec3<f32>(1.0, 1.0, 1.0);
}

