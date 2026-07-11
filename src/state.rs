use core::f32;
use std::{sync::Arc, time::Instant};

use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor, BufferUsages, ComputePassDescriptor, util::{BufferInitDescriptor, DeviceExt}};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{camera::Camera, init_shape::InitShapeUniforms, matrix4::Matrix4, particle_chunk::ParticleChunk, particle_vertex::ParticleVertex, quaternion::Quaternion, render_uniforms::RenderUniforms, simulation_parameters::{self, SimulationParameters}, texture::Texture, vector::Vec3};



// This will store the state of our game
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    depth_texture: Texture,
    render_pipeline: wgpu::RenderPipeline,
    render_uniform_bind_group: wgpu::BindGroup,
    render_uniform_buffer: wgpu::Buffer,

    update_particle_compute_pipeline: wgpu::ComputePipeline,
    particle_init_cube_pipeline: wgpu::ComputePipeline,
    particle_init_uniform_buffer: wgpu::Buffer,
    particle_init_bind_group: wgpu::BindGroup,

    particle_init_sphere_pipeline: wgpu::ComputePipeline,

    particle_chunks: Vec<ParticleChunk>,
    simulation_uniform_buffer: wgpu::Buffer,
    simulation_uniform_bind_group: wgpu::BindGroup,

    simulation_parameters: SimulationParameters,
    particle_count: u32,

    camera: Camera,
    last_frame_time: Instant,
    last_cursor_position: (f32, f32),
}

impl State {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>, particle_count: u32) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let camera = Camera::new(Vec3::new3([0.0, 0.0, 0.0]), Quaternion::from_vec(Vec3::new3([0.0, 0.0, 1.0]), Vec3::new3([0.0, 1.0, 0.0])), 50.0f32.to_radians(), size.width as f32 / size.height as f32, 0.001, 3000.0);

        
        let simulation_parameters = SimulationParameters {
            gravity_position: Vec3::new3([0.0, 0.0, 0.0]),
            gravity_strength: 3.0,
            _padding: 0.0,
            _padding2: 0.0,
            _padding3: 0.0,
        };

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
    
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let render_shader = device.create_shader_module(wgpu::include_wgsl!("render.wgsl"));

        let render_uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Render Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[Some(&render_uniform_bind_group_layout)],
                immediate_size: 0,
            });

        let render_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Render Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera.get_render_uniforms()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let render_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Render Uniform Bind Group"),
            layout: &render_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_uniform_buffer.as_entire_binding(),
                }
            ]
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[ParticleVertex::desc()], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }), // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None, // 6.
        });

        let update_particle_shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        let simulation_uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Simulation Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let simulation_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Simulation Uniform Buffer"),
            contents: bytemuck::cast_slice(&[simulation_parameters]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let simulation_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Simulation Uniform Bind Group"),
            layout: &simulation_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: simulation_uniform_buffer.as_entire_binding(),
                }
            ]
        });

        let compute_buffers_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Compute Buffers Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ]
        });

        let update_particle_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Update Compute Pipeline Layout"),
            bind_group_layouts: &[
                Some(&compute_buffers_bind_group_layout),
                Some(&simulation_uniform_bind_group_layout)
            ],
            immediate_size: 0,
        });

        let update_particle_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle update compute pipeline"),
            layout: Some(&update_particle_pipeline_layout),
            module: &update_particle_shader,
            entry_point: Some("update_particle"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None
        });

        let mut particle_chunks = Vec::new();
        let mut particles_to_init = particle_count;
        while particles_to_init != 0 {
            let chunk_size = std::cmp::min(particles_to_init, device.limits().max_compute_workgroups_per_dimension);
            particle_chunks.push(ParticleChunk::new(&device, &compute_buffers_bind_group_layout, chunk_size));
            particles_to_init -= chunk_size;
        }

        println!("Created {} particle chunks", particle_chunks.len());

        let particle_init_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Particle Init Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let init_particle_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Init Compute Pipeline Layout"),
            bind_group_layouts: &[
                Some(&compute_buffers_bind_group_layout),
                Some(&particle_init_bind_group_layout)
            ],
            immediate_size: 0,
        });

        let init_cube_particle_shader = device.create_shader_module(wgpu::include_wgsl!("init_cube.wgsl"));

        let particle_init_cube_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Init Cube Compute Pipeline"),
            layout: Some(&init_particle_pipeline_layout),
            module: &init_cube_particle_shader,
            entry_point: Some("init_cube"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None
        });

        let init_sphere_particle_shader = device.create_shader_module(wgpu::include_wgsl!("init_sphere.wgsl"));
        let particle_init_sphere_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Particle Init Sphere Compute Pipeline"),
            layout: Some(&init_particle_pipeline_layout),
            module: &init_sphere_particle_shader,
            entry_point: Some("init_sphere"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None
        });

        let particle_init_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Particle Init Uniform Buffer"),
            contents: bytemuck::cast_slice(&[InitShapeUniforms {
                spawn_density: 0u32,
                current_particle_offset: 0u32,
                size: 0.0f32
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let particle_init_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Particle Init Bind Group"),
            layout: &particle_init_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_init_uniform_buffer.as_entire_binding(),
                }
            ]
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured: false,
            window,
            depth_texture,
            render_pipeline,
            render_uniform_bind_group,
            render_uniform_buffer,
            update_particle_compute_pipeline,
            particle_count,
            particle_chunks,
            particle_init_cube_pipeline,
            particle_init_sphere_pipeline,
            particle_init_uniform_buffer,
            particle_init_bind_group,

            camera,
            last_frame_time: Instant::now(),
            simulation_parameters,
            simulation_uniform_buffer,
            simulation_uniform_bind_group,
            last_cursor_position: (0.0, 0.0),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            //let max = 2048;

            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
            self.camera.set_aspect_ratio(width as f32 / height as f32);
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_frame_time).as_secs_f32();
        self.camera.update(delta_time);
        self.simulation_parameters.gravity_position.w = delta_time;
        self.queue.write_buffer(&self.simulation_uniform_buffer, 0, bytemuck::cast_slice(&[self.simulation_parameters]));
        self.last_frame_time = now;
    }
    
    pub fn render(&mut self) -> anyhow::Result<()> {
        let start = Instant::now();
        self.update();
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
            
        let output = match self.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
                wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                    self.surface.configure(&self.device, &self.config);
                    surface_texture
                }
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Validation => {
                    // Skip this frame
                    return Ok(());
                }
                wgpu::CurrentSurfaceTexture::Outdated => {
                    self.surface.configure(&self.device, &self.config);
                    return Ok(());
                }
                wgpu::CurrentSurfaceTexture::Lost => {
                    // You could recreate the devices and all resources
                    // created with it here, but we'll just bail
                    anyhow::bail!("Lost device");
                }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.queue.write_buffer(&self.render_uniform_buffer, 0, bytemuck::cast_slice(&[self.camera.get_render_uniforms()]));

        let mut load_ops = (wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }), wgpu::LoadOp::Clear(1.0));
        for particle_chunk in &self.particle_chunks {
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Particle Update Compute Pass"),
                    timestamp_writes: None
                });

                compute_pass.set_pipeline(&self.update_particle_compute_pipeline);
                compute_pass.set_bind_group(1, &self.simulation_uniform_bind_group, &[]);
                particle_chunk.dispatch_update(&mut compute_pass);
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[
                        // This is what @location(0) in the fragment shader targets
                        Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: load_ops.0,
                                store: wgpu::StoreOp::Store,
                            }
                        })
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: load_ops.1,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, particle_chunk.get_vertex_buffer().slice(..));
                render_pass.draw(0..particle_chunk.get_particle_count(), 0..1);
            }
            load_ops = (wgpu::LoadOp::Load, wgpu::LoadOp::Load);
        }



        self.queue.submit([encoder.finish()]);
        output.present();

        let end = Instant::now();
        let frame_time = (end - start).as_nanos() as f32 / 1_000_000f32;
        self.window.set_title(format!("ParticleSystem | {:>9.3} ms, {}/s", frame_time, 1000.0 / frame_time).as_str());
        Ok(())

    }

    pub fn init_particles_as_cube(&self) {
        
        let mut current_offset = 0u32;
        for particle_chunk in &self.particle_chunks {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Init Cube Encoder"),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Init Cube Compute Pass"),
                    timestamp_writes: None
                });

                self.queue.write_buffer(&self.particle_init_uniform_buffer, 0, bytemuck::cast_slice(&[InitShapeUniforms {
                    spawn_density: 9000u32,
                    current_particle_offset: current_offset,
                    size: 0.01
                }]));
                compute_pass.set_pipeline(&self.particle_init_cube_pipeline);
                compute_pass.set_bind_group(1, &self.particle_init_bind_group, &[]);
                particle_chunk.dispatch_update(&mut compute_pass);
                current_offset += particle_chunk.get_particle_count();
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    pub fn init_particles_as_sphere(&self) {
        let mut current_offset = 0u32;
        for particle_chunk in &self.particle_chunks {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Init Sphere Encoder"),
            });
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Init Sphere Compute Pass"),
                    timestamp_writes: None
                });

                self.queue.write_buffer(&self.particle_init_uniform_buffer, 0, bytemuck::cast_slice(&[InitShapeUniforms {
                    spawn_density: self.particle_count,
                    current_particle_offset: current_offset,
                    size: 0.1
                }]));
                compute_pass.set_pipeline(&self.particle_init_sphere_pipeline);
                compute_pass.set_bind_group(1, &self.particle_init_bind_group, &[]);
                particle_chunk.dispatch_update(&mut compute_pass);
                current_offset += particle_chunk.get_particle_count();
            }
            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    // impl State
    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::Digit1, true) => {
                self.init_particles_as_cube();
            },
            (KeyCode::Digit2, true) => {
                self.init_particles_as_sphere();
            },
            _ => {
                self.camera.handle_input(code, is_pressed);
            }
        }
    }

    pub fn handle_absolute_mouse_position(&mut self, x: f64, y: f64) {
        self.last_cursor_position = (x as f32, y as f32);
    }

    pub fn handle_mouse_move(&mut self, delta_x: f64, delta_y: f64) {
        self.camera.handle_mouse_movement(delta_x as f32, delta_y as f32);
    }

    pub fn handle_mouse_button(&mut self, button: winit::event::MouseButton, is_pressed: bool) {
        match button {
            winit::event::MouseButton::Right => {
                self.camera.handle_mouse_button(button, is_pressed);
            }
            winit::event::MouseButton::Left => {
                if is_pressed {
                    // set gravity center under mouse cursor depending on the last cursor position and the camera's view and projection matrices
                    let (x, y) = self.last_cursor_position;
                    self.camera.get_direction_from_screen_coordinates(x, y, self.config.width as f32, self.config.height as f32).map(|dir| {
                        let gravity_position = self.camera.get_position() + -dir * 6.0;
                        self.simulation_parameters.gravity_position = gravity_position;
                    });
                }
            }
            _ => {}
        }
    }

}