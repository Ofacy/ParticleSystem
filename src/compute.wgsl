
struct ParticleLifetime {
    @location(0) velocity: vec3<f32>,
    @location(1) lifetime: f32,
}
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
var<storage, read_write> particle_lifetimes: array<ParticleLifetime>;

@group(0) @binding(1)
var<storage, read_write> particle_vertices: array<ParticleVertex>;

@group(1) @binding(0)
var<uniform> simulation_uniforms: SimulationUniforms;

fn random_in_sphere(seed: vec3<f32>) -> vec3<f32> {
    let u = fract(sin(dot(seed, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
    let v = fract(sin(dot(seed + 1.0, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
    let theta = u * 2.0 * 3.14159265358979323846;
    let phi = acos(2.0 * v - 1.0);
    let x = sin(phi) * cos(theta);
    let y = sin(phi) * sin(theta);
    let z = cos(phi);
    return vec3<f32>(x, y, z);
}

@compute
@workgroup_size(64, 1, 1)
fn update_particle(
    @builtin(global_invocation_id) global_invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {
    let workgroup_size = vec3<u32>(64, 1, 1);
    let particle_index: u32 =
        global_invocation_id.x +
        (global_invocation_id.y * workgroup_size.x * num_workgroups.x) +
        (global_invocation_id.z * workgroup_size.x * workgroup_size.y * num_workgroups.x * num_workgroups.y);
    let gravity_direction = simulation_uniforms.gravity_position - particle_vertices[particle_index].position;
    let gravity_distance = length(gravity_direction);
    let lifetime = particle_lifetimes[particle_index].lifetime - simulation_uniforms.delta_time;
    if (lifetime < 0.0) {
        particle_lifetimes[particle_index].lifetime = simulation_uniforms.starting_lifetime;
        particle_vertices[particle_index].position = simulation_uniforms.starting_position + random_in_sphere(vec3<f32>(f32(particle_index * 1u), f32(particle_index * 2u), f32(particle_index * 3u))) * simulation_uniforms.starting_position_radius;

        particle_lifetimes[particle_index].velocity = random_in_sphere(vec3<f32>(f32(particle_index * 4u), f32(particle_index * 5u), f32(particle_index * 6u))) * 0.3;
        //particle_lifetimes[particle_index].velocity = vec3<f32>(0.0, 0.0, 420.0);
        return;
    }
    particle_lifetimes[particle_index].lifetime = lifetime;
    particle_lifetimes[particle_index].velocity += gravity_direction * simulation_uniforms.gravity_strength * simulation_uniforms.delta_time / (gravity_distance * gravity_distance);
    particle_vertices[particle_index].position += particle_lifetimes[particle_index].velocity * simulation_uniforms.delta_time;
}

