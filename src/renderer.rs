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

        let char_list: Vec<char> = "o: sin 440 >> mul 0.1".chars().collect();

        let (render_pipeline, vertex_buffer, index_buffer, num_indices, diffuse_bind_group) =
            crate::utils::update_renderer(&window, &device, &config, &queue, &char_list, &font);

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

            let (render_pipeline, vertex_buffer, index_buffer, num_indices, diffuse_bind_group) =
                crate::utils::update_renderer(
                    &self.window,
                    &self.device,
                    &self.config,
                    &self.queue,
                    &self.char_list,
                    &self.font,
                );
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
