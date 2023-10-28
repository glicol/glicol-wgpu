use std::{cell::RefCell, iter, rc::Rc};

use guillotiere::{AtlasAllocator, Size};
use wgpu::{util::DeviceExt, CommandEncoderDescriptor};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn rasterize_character(ch: char, scale: f32, font: &fontdue::Font) -> (Vec<u8>, usize, usize) {
    // let scale = 32.0;
    let (metrics, bitmap) = font.rasterize(ch, scale);
    (bitmap, metrics.width as usize, metrics.height as usize)
}

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    // position_buffer: wgpu::Buffer,
    num_indices: u32,
    position: f32,
    // #[allow(dead_code)]
    // diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    // position_bind_group: wgpu::BindGroup,
    window: Rc<RefCell<winit::window::Window>>,
    char_list: Vec<char>,
    font: fontdue::Font,
    audio_engine: Option<Rc<RefCell<glicol::Engine<128>>>>,
    bpm: f32,
}

impl Renderer {
    pub async fn new(window: Rc<RefCell<winit::window::Window>>) -> Self {
        #[cfg(target_arch = "wasm32")]
        console_log::init_with_level(log::Level::Warn).expect("cannot init logger");

        let size = window.borrow().inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&*window.borrow()) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
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

        let b = include_bytes!("FiraCode-Regular.ttf") as &[u8];
        let font = fontdue::Font::from_bytes(b, fontdue::FontSettings::default()).unwrap();

        let char_list: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
        let mut allocator = AtlasAllocator::new(Size::new(2048, 2048));
        let font_size = 32.0 * window.borrow().scale_factor() as f32 * 96. / 72.;
        log::warn!("scale_factor: {}", window.borrow().scale_factor());
        let padding = 10;
        allocator.clear();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Font Texture Atlas"),
            size: wgpu::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut line_shift = 0.0; // shift caused by \n character
        let mut x_offset = 0.;
        let mut bypass_count = 0; // \n is not rendered, so we need to skip it

        for (i, ch) in char_list.iter().enumerate() {
            let (metrics, bitmap) = font.rasterize(*ch, font_size);

            // tracing::warn!("\n\n {:?}, Metrics {:?}\n\n", ch, metrics);
            let size = Size::new(
                metrics.width as i32 + padding * 2,
                metrics.height as i32 + padding * 2,
            );

            let char_width = metrics.width as f32 / window.borrow().inner_size().width as f32;
            let char_height = (metrics.height as f32) / window.borrow().inner_size().height as f32;
            let y_offset =
                metrics.ymin as f32 / window.borrow().inner_size().height as f32 - line_shift;
            let font_size_scale = 32.0 / window.borrow().inner_size().height as f32;

            if let Some(allocation) = allocator.allocate(size) {
                // tracing::warn!("\n\n allocation.rectangle {:?}\n\n", allocation.rectangle);

                let encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Texture Upload Encoder"),
                });
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: allocation.rectangle.min.x as u32 + padding as u32,
                            y: allocation.rectangle.min.y as u32 + padding as u32,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bitmap,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(metrics.width as u32),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: metrics.width as u32,
                        height: metrics.height as u32,
                        depth_or_array_layers: 1,
                    },
                );
                queue.submit(Some(encoder.finish()));
                let top_left_x = (allocation.rectangle.min.x + padding) as f32 / 2048.;
                let top_left_y = (allocation.rectangle.min.y + padding) as f32 / 2048.;
                let bottom_right_x = (allocation.rectangle.max.x - padding) as f32 / 2048.;
                let bottom_right_y = (allocation.rectangle.max.y - padding) as f32 / 2048.;
                vertices.extend_from_slice(&[
                    Vertex {
                        position: [
                            -1.0 + x_offset,
                            char_height + y_offset + 1.0 - font_size_scale,
                            0.0,
                        ],
                        tex_coords: [top_left_x, top_left_y],
                    },
                    Vertex {
                        position: [-1.0 + x_offset, y_offset + 1.0 - font_size_scale, 0.0],
                        tex_coords: [top_left_x, bottom_right_y],
                    },
                    Vertex {
                        position: [
                            char_width + x_offset - 1.0,
                            y_offset + 1.0 - font_size_scale,
                            0.0,
                        ],
                        tex_coords: [bottom_right_x, bottom_right_y],
                    },
                    Vertex {
                        position: [
                            char_width + x_offset - 1.0,
                            char_height + y_offset + 1.0 - font_size_scale,
                            0.0,
                        ],
                        tex_coords: [bottom_right_x, top_left_y],
                    },
                ]);
                indices.extend_from_slice(&[
                    0 + i as u16 * 4,
                    1 + i as u16 * 4,
                    2 + i as u16 * 4,
                    2 + i as u16 * 4,
                    3 + i as u16 * 4,
                    0 + i as u16 * 4,
                ]);
                x_offset += metrics.advance_width / window.borrow().inner_size().width as f32
            } else {
                tracing::warn!("allocation failed");
            }
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = indices.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            window,
            position: 0.0,
            char_list,
            font,
            audio_engine: None,
            bpm: 120.,
        }
    }

    pub fn window(&self) -> &Rc<RefCell<Window>> {
        &self.window
    }

    #[cfg(target_arch = "wasm32")]
    pub fn add_audio_engine(&mut self, engine: Rc<RefCell<glicol::Engine<128>>>) {
        self.audio_engine = Some(engine);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            let mut allocator = AtlasAllocator::new(Size::new(2048, 2048));
            let font_size = 32.0 * self.window.borrow().scale_factor() as f32 * 96. / 72.;
            let padding = 10; // (10. * self.window.borrow().scale_factor()) as i32;
            allocator.clear();
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Font Texture Atlas"),
                size: wgpu::Extent3d {
                    width: 2048,
                    height: 2048,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            let mut line_shift = 0.0; // shift caused by \n character
            let mut x_offset = 0.;
            let mut bypass_count = 0; // \n is not rendered, so we need to skip it

            for (i, ch) in self.char_list.iter().enumerate() {
                let (metrics, bitmap) = self.font.rasterize(*ch, font_size);

                // tracing::warn!("\n\n {:?}, Metrics {:?}\n\n", ch, metrics);
                let size = Size::new(
                    metrics.width as i32 + padding * 2,
                    metrics.height as i32 + padding * 2,
                );

                let char_width =
                    metrics.width as f32 / self.window.borrow().inner_size().width as f32;
                let char_height =
                    (metrics.height as f32) / self.window.borrow().inner_size().height as f32;
                let y_offset = metrics.ymin as f32
                    / self.window.borrow().inner_size().height as f32
                    - line_shift;
                let font_size_scale = font_size / self.window.borrow().inner_size().height as f32;

                if let Some(allocation) = allocator.allocate(size) {
                    // tracing::warn!("\n\n allocation.rectangle {:?}\n\n", allocation.rectangle);

                    let encoder = self
                        .device
                        .create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("Texture Upload Encoder"),
                        });
                    self.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: allocation.rectangle.min.x as u32 + padding as u32,
                                y: allocation.rectangle.min.y as u32 + padding as u32,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &bitmap,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(metrics.width as u32),
                            rows_per_image: None,
                        },
                        wgpu::Extent3d {
                            width: metrics.width as u32,
                            height: metrics.height as u32,
                            depth_or_array_layers: 1,
                        },
                    );
                    self.queue.submit(Some(encoder.finish()));
                    let top_left_x = (allocation.rectangle.min.x + padding) as f32 / 2048.;
                    let top_left_y = (allocation.rectangle.min.y + padding) as f32 / 2048.;
                    let bottom_right_x = (allocation.rectangle.max.x - padding) as f32 / 2048.;
                    let bottom_right_y = (allocation.rectangle.max.y - padding) as f32 / 2048.;
                    vertices.extend_from_slice(&[
                        Vertex {
                            position: [
                                -1.0 + x_offset,
                                char_height + y_offset + 1.0 - font_size_scale,
                                0.0,
                            ],
                            tex_coords: [top_left_x, top_left_y],
                        },
                        Vertex {
                            position: [-1.0 + x_offset, y_offset + 1.0 - font_size_scale, 0.0],
                            tex_coords: [top_left_x, bottom_right_y],
                        },
                        Vertex {
                            position: [
                                char_width + x_offset - 1.0,
                                y_offset + 1.0 - font_size_scale,
                                0.0,
                            ],
                            tex_coords: [bottom_right_x, bottom_right_y],
                        },
                        Vertex {
                            position: [
                                char_width + x_offset - 1.0,
                                char_height + y_offset + 1.0 - font_size_scale,
                                0.0,
                            ],
                            tex_coords: [bottom_right_x, top_left_y],
                        },
                    ]);
                    indices.extend_from_slice(&[
                        0 + i as u16 * 4,
                        1 + i as u16 * 4,
                        2 + i as u16 * 4,
                        2 + i as u16 * 4,
                        3 + i as u16 * 4,
                        0 + i as u16 * 4,
                    ]);
                    x_offset +=
                        metrics.advance_width / self.window.borrow().inner_size().width as f32
                } else {
                    tracing::warn!("allocation failed");
                }
            }

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let texture_bind_group_layout =
                self.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                        ],
                        label: Some("texture_bind_group_layout"),
                    });

            let diffuse_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

            let shader = self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                });

            let render_pipeline_layout =
                self.device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[&texture_bind_group_layout],
                        push_constant_ranges: &[],
                    });

            let render_pipeline =
                self.device
                    .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[Vertex::desc()],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState {
                                format: self.config.format,
                                blend: Some(wgpu::BlendState {
                                    color: wgpu::BlendComponent::REPLACE,
                                    alpha: wgpu::BlendComponent::REPLACE,
                                }),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: Some(wgpu::Face::Back),
                            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                            // or Features::POLYGON_MODE_POINT
                            polygon_mode: wgpu::PolygonMode::Fill,
                            // Requires Features::DEPTH_CLIP_CONTROL
                            unclipped_depth: false,
                            // Requires Features::CONSERVATIVE_RASTERIZATION
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: 1,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        // If the pipeline will be used with a multiview render pass, this
                        // indicates how many array layers the attachments will have.
                        multiview: None,
                    });

            let vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            let num_indices = indices.len() as u32;
            self.render_pipeline = render_pipeline;
            self.vertex_buffer = vertex_buffer;
            self.index_buffer = index_buffer;
            self.num_indices = num_indices;
            self.diffuse_bind_group = diffuse_bind_group;
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.request_position_change(event)
    }

    fn request_position_change(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Left => {
                        if is_pressed {
                            self.position -= 0.01;
                        }
                        #[cfg(target_arch = "wasm32")]
                        log::warn!("position: {}", self.position);
                        if let Some(engine) = &self.audio_engine {
                            let mut engine_borrow = engine.borrow_mut();
                            self.bpm -= 10.;
                            engine_borrow.set_bpm(self.bpm);
                            // engine_borrow
                            //     .update_with_code(r#"o: speed 4.0 >> seq _ 60 >> sn 0.05"#);
                        }
                        true
                    }

                    VirtualKeyCode::Right => {
                        if is_pressed {
                            self.position += 0.01;
                        }
                        #[cfg(target_arch = "wasm32")]
                        log::warn!("position: {}", self.position);
                        if let Some(engine) = &self.audio_engine {
                            let mut engine_borrow = engine.borrow_mut();
                            self.bpm += 10.;
                            engine_borrow.set_bpm(self.bpm);
                            // engine_borrow.update_with_code(r#"o: speed 4.0 >> seq 60 >> bd 0.03"#);
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        // self.queue.write_buffer(
        //     &self.position_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.position]),
        // );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            // render_pass.set_bind_group(1, &self.position_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
// pub async fn run() {
//     // env_logger::init();

//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new().build(&event_loop).unwrap();

//     // State::new uses async code, so we're going to wait for it to finish
//     let mut state = State::new(window).await;

//     event_loop.run(move |event, _, control_flow| {
//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == state.window().id() => {
//                 if !state.input(event) {
//                     match event {
//                         WindowEvent::CloseRequested
//                         | WindowEvent::KeyboardInput {
//                             input:
//                                 KeyboardInput {
//                                     state: ElementState::Pressed,
//                                     virtual_keycode: Some(VirtualKeyCode::Escape),
//                                     ..
//                                 },
//                             ..
//                         } => *control_flow = ControlFlow::Exit,
//                         WindowEvent::Resized(physical_size) => {
//                             state.resize(*physical_size);
//                         }
//                         WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                             // new_inner_size is &mut so w have to dereference it twice
//                             state.resize(**new_inner_size);
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//             Event::RedrawRequested(window_id) if window_id == state.window().id() => {
//                 state.update();
//                 state.render().unwrap();
//             }
//             Event::MainEventsCleared => {
//                 // RedrawRequested will only trigger once, unless we manually
//                 // request it.
//                 state.window().request_redraw();
//             }
//             _ => {}
//         }
//     });
// }
