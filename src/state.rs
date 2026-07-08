use core::f32;
use std::{mem, sync::Arc, time::Instant};

use wgpu::{BindGroupDescriptor, BindGroupLayoutDescriptor, BindingType, BufferDescriptor, BufferUsages, ComputePassDescriptor, ShaderStages, util::{BufferInitDescriptor, DeviceExt}, wgc::validation::BindingTypeName::Buffer};
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{particle_lifetime::ParticleLifetime, particle_vertex::ParticleVertex};



// This will store the state of our game
pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,

    update_particle_compute_pipeline: wgpu::ComputePipeline,
    particle_init_cube_pipeline: wgpu::ComputePipeline,
    particle_init_uniform_buffer: wgpu::Buffer,
    particle_init_bind_group: wgpu::BindGroup,

    lifetime_buffer: wgpu::Buffer,
    compute_bind_group: wgpu::BindGroup,
    
    vertex_buffer: wgpu::Buffer,

    particle_count: u32
}

impl State {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(window: Arc<Window>, particle_count: u32) -> anyhow::Result<Self> {
        let size = window.inner_size();

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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

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
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None, // 6.
        });

        let update_particle_shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

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
                Some(&compute_buffers_bind_group_layout)
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

        let lifetime_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Lifetime Buffer"),
            usage: BufferUsages::STORAGE,
            size: particle_count as u64 * mem::size_of::<ParticleLifetime>() as u64,
            mapped_at_creation: false
        });

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particle Vertex Buffer"),
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
            size: particle_count as u64 * mem::size_of::<ParticleVertex>() as u64, 
            mapped_at_creation: false
        });

        let compute_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &update_particle_compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: lifetime_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vertex_buffer.as_entire_binding(),
                },
            ]
        });

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

        let particle_init_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Particle Init Uniform Buffer"),
            contents: bytemuck::cast_slice(&[0.01f32]),
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
            render_pipeline,
            update_particle_compute_pipeline,
            lifetime_buffer,
            vertex_buffer,
            particle_count,
            compute_bind_group,
            particle_init_cube_pipeline,
            particle_init_uniform_buffer,
            particle_init_bind_group
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            //let max = 2048;

            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn update(&mut self) {

    }
    
    pub fn render(&mut self) -> anyhow::Result<()> {
        let start = Instant::now();
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

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Particle Compute Pass"),
                timestamp_writes: None
            });

            compute_pass.set_pipeline(&self.update_particle_compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.particle_init_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.particle_count, 1, 1);
        }

        {
            // 1.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }
                            ),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline); // 2.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.particle_count, 0..1);
        }


        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let end = Instant::now();
        let frame_time = (end - start).as_nanos() as f32 / 1_000_000f32;
        self.window.set_title(format!("ParticleSystem | {:>9.3} ms", frame_time).as_str());
        Ok(())

    }

    pub fn init_particles_as_cube(&self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Init Cube Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Init Cube Compute Pass"),
                timestamp_writes: None
            });

            compute_pass.set_pipeline(&self.particle_init_cube_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.set_bind_group(1, &self.particle_init_bind_group, &[]);
            compute_pass.dispatch_workgroups(self.particle_count, 1, 1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    // impl State
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            (KeyCode::Digit1, true) => {
                self.init_particles_as_cube();
            }
            _ => {}
        }
    }

}