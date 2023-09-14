use super::binding_structs::*;
use winit::{window::Window, event::WindowEvent};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
    pub info: Info,
    info_buffer: wgpu::Buffer,
    geometry: Vec<Geometry>
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window, info: Info) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        println!("{:?}", adapter.get_info());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None, // Trace path
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())            
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None
                    },
                    count: None
                },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 1,
                //     visibility: wgpu::ShaderStages::FRAGMENT,
                //     ty: wgpu::BindingType::Buffer { 
                //         ty: wgpu::BufferBindingType::Uniform, 
                //         has_dynamic_offset: false, 
                //         min_binding_size: None
                //     },
                //     count: None
                // },
                // wgpu::BindGroupLayoutEntry {
                //     binding: 2,
                //     visibility: wgpu::ShaderStages::FRAGMENT,
                //     ty: wgpu::BindingType::Buffer { 
                //         ty: wgpu::BufferBindingType::Uniform, 
                //         has_dynamic_offset: false, 
                //         min_binding_size: None
                //     },
                //     count: None
                // }
            ]
        });

        let info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("info buffer"),
            size: info.get_bytes().len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor { 
            label: Some("Vertex Buffer"), 
            size: 12 * 4, 
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, 
            mapped_at_creation: false
        });

        queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice::<f32, u8>(&[
        //    X     Y    Square because I'm not using vertices
            -1.0, -1.0,
             1.0, -1.0,
             1.0,  1.0,

            -1.0, -1.0,
            -1.0,  1.0,
             1.0,  1.0
        ]));

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 8,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0
                }
            ]
        };

        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        Self { 
            surface, device, queue, config, size, window, 
            render_pipeline, vertex_buffer, bind_group_layout, bind_group: None,
            info, info_buffer, geometry: Vec::new()
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = winit::dpi::PhysicalSize::new(new_size.width / 2, new_size.height / 2);
            self.config.width = new_size.width/2;
            self.config.height = new_size.height/2;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {
        self.info.set_sizes(self.size.width as f32, self.size.height as f32);

        self.queue.write_buffer(&self.info_buffer, 0, &self.info.get_bytes()[..]);

        // let geometry_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
        //     label: Some("info buffer"),
        //     size: 4 * 2 * self.geometry.len() as u64,
        //     usage: wgpu::BufferUsages::UNIFORM,
        //     mapped_at_creation: true
        // });

        // self.queue.write_buffer(
        //     &geometry_buffer, 
        //     0, 
        //     &self.geometry
        //         .iter()
        //         .map(|geometry| geometry.get_bytes())
        //         .flatten()
        //         .collect::<Vec<u8>>()[..]
        // );

        self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.info_buffer,
                        offset: 0,
                        size: None
                    })
                },
                // wgpu::BindGroupEntry {
                //     binding: 1,
                //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                //         buffer: &geometry_buffer,
                //         offset: 0,
                //         size: None
                //     })
                // }
            ]
        }));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6,1..2);

        drop(render_pass);
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}