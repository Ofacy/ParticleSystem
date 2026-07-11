
struct ParticleLifetime {
    @location(0) velocity: vec3<f32>,
    @location(1) lifetime: f32,
}
struct ParticleVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

struct InitCubeUniforms {
    current_particle_offset: u32,
    spawn_density: u32,
    size: f32,
}

@group(0) @binding(0)
var<storage, read_write> particle_lifetimes: array<ParticleLifetime>;

@group(0) @binding(1)
var<storage, read_write> particle_vertices: array<ParticleVertex>;

@group(1) @binding(0)
var<uniform> init_cube_uniforms: InitCubeUniforms;

fn get_particle_count_per_axis() -> u32 {
    return u32(sqrt(f32(init_cube_uniforms.spawn_density)));
}

fn get_jitter_offset(index: u32, particle_count_per_axis: u32, size: f32) -> f32 {
    let jitter_strength = size / f32(particle_count_per_axis);
    let random_value = fract(sin(f32(index) * 12.9898) * 43758.5453);
    return (random_value - 0.5) * jitter_strength;
}

@compute
@workgroup_size(64)
fn init_cube(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>
) {
    let real_index = global_invocation_id.x + init_cube_uniforms.current_particle_offset;
    let particle_count_per_axis = get_particle_count_per_axis();
    var position = vec3<f32>(
        
        f32(real_index % particle_count_per_axis) * init_cube_uniforms.size - (init_cube_uniforms.size * f32(particle_count_per_axis) / 2.0) + get_jitter_offset(real_index, particle_count_per_axis, init_cube_uniforms.size),
        f32((real_index / particle_count_per_axis) % particle_count_per_axis) * init_cube_uniforms.size - (init_cube_uniforms.size * f32(particle_count_per_axis) / 2.0) + get_jitter_offset(real_index, particle_count_per_axis, init_cube_uniforms.size),
        f32(real_index / (particle_count_per_axis * particle_count_per_axis)) * init_cube_uniforms.size - (init_cube_uniforms.size * f32(particle_count_per_axis) / 2.0) + get_jitter_offset(real_index, particle_count_per_axis, init_cube_uniforms.size)
    );
    particle_lifetimes[global_invocation_id.x].velocity = vec3<f32>(0.0, 0.0, 0.0);
    particle_lifetimes[global_invocation_id.x].lifetime = 0.0;
    particle_vertices[global_invocation_id.x].position = position;
    particle_vertices[global_invocation_id.x].color = vec3<f32>(1.0, 1.0, 1.0);
}

