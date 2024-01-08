use std::{cell::RefCell, iter, rc::Rc};

// use guillotiere::{AtlasAllocator, Size};
use hashbrown::HashSet;
// use wgpu::{util::DeviceExt, CommandEncoderDescriptor};
use winit::{
    event::*,
    // event_loop::{ControlFlow, EventLoop},
    window::Window, //WindowBuilder
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::audio::run_audio;
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

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
    // position: f32,
    // #[allow(dead_code)]
    // diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    // position_bind_group: wgpu::BindGroup,
    window: Rc<RefCell<winit::window::Window>>,
    char_list: Vec<char>,
    font: fontdue::Font,
    // #[cfg(target_arch = "wasm32")]
    // audio_engine: Option<Rc<RefCell<glicol::Engine<128>>>>,
    // #[cfg(target_arch = "wasm32")]
    // bpm: f32,
    cursors: Vec<usize>,
    modifiers: HashSet<VirtualKeyCode>,
    shared_string: std::sync::Arc<std::sync::Mutex<String>>,
    has_update: std::sync::Arc<std::sync::atomic::AtomicBool>,
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

        let char_list: Vec<char> = include_str!("./code.glicol").chars().collect();
        let cursors = vec![0];
        let (render_pipeline, vertex_buffer, index_buffer, num_indices, diffuse_bind_group) =
            crate::utils::update_renderer(
                &window, &device, &config, &queue, &char_list, &cursors, &font,
            );

        let host = cpal::default_host();
        let audio_device = host.default_output_device().unwrap();
        let audio_config = audio_device.default_output_config().unwrap();

        let code = String::from("");
        let shared_string = std::sync::Arc::new(std::sync::Mutex::new(code));
        let shared_string_clone = shared_string.clone();
        let has_update = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let has_update_clone = has_update.clone();

        let _audio_thread = std::thread::spawn(move || {
            // let options = (
            //     ptr_rb_left_clone,
            //     ptr_rb_right_clone,
            //     index_clone,
            //     samples_l_ptr_clone,
            //     samples_r_ptr_clone,
            //     samples_index_clone,
            //     path,
            //     bpm,
            //     capacity_clone,
            // );
            let options = (shared_string_clone, has_update_clone);
            match audio_config.sample_format() {
                cpal::SampleFormat::I8 => {
                    run_audio::<i8>(&audio_device, &audio_config.into(), options)
                }
                cpal::SampleFormat::I16 => {
                    run_audio::<i16>(&audio_device, &audio_config.into(), options)
                }
                // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
                cpal::SampleFormat::I32 => {
                    run_audio::<i32>(&audio_device, &audio_config.into(), options)
                }
                // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
                cpal::SampleFormat::I64 => {
                    run_audio::<i64>(&audio_device, &audio_config.into(), options)
                }
                cpal::SampleFormat::U8 => {
                    run_audio::<u8>(&audio_device, &audio_config.into(), options)
                }
                cpal::SampleFormat::U16 => {
                    run_audio::<u16>(&audio_device, &audio_config.into(), options)
                }
                // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
                cpal::SampleFormat::U32 => {
                    run_audio::<u32>(&audio_device, &audio_config.into(), options)
                }
                // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
                cpal::SampleFormat::U64 => {
                    run_audio::<u64>(&audio_device, &audio_config.into(), options)
                }
                cpal::SampleFormat::F32 => {
                    run_audio::<f32>(&audio_device, &audio_config.into(), options)
                }
                cpal::SampleFormat::F64 => {
                    run_audio::<f64>(&audio_device, &audio_config.into(), options)
                }
                sample_format => panic!("Unsupported sample format '{sample_format}'"),
            }
        });

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
            // position: 0.0,
            char_list,
            font,
            // #[cfg(target_arch = "wasm32")]
            // audio_engine: None,
            // #[cfg(target_arch = "wasm32")]
            // bpm: 120.,
            cursors,
            modifiers: HashSet::new(),
            shared_string,
            has_update,
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

            self.update();
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // #[cfg(target_arch = "wasm32")]
        if self.update_code(event) {
            return true;
        }

        if self.detect_modifiers(event) {
            return true;
        } else if self.move_cursor(event) {
            self.update();
            return true;
        } else if self.input_or_delete_character(event) {
            self.update();
            return true;
        } else {
            return false;
        }
    }

    pub fn detect_modifiers(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if keycode == &VirtualKeyCode::RAlt
                    || keycode == &VirtualKeyCode::LAlt
                    || keycode == &VirtualKeyCode::RControl
                    || keycode == &VirtualKeyCode::LControl
                    || keycode == &VirtualKeyCode::RShift
                    || keycode == &VirtualKeyCode::LShift
                    || keycode == &VirtualKeyCode::RWin
                    || keycode == &VirtualKeyCode::LWin
                {
                    self.modifiers.insert(*keycode);
                    true
                } else {
                    false
                }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if keycode == &VirtualKeyCode::RAlt
                    || keycode == &VirtualKeyCode::LAlt
                    || keycode == &VirtualKeyCode::RControl
                    || keycode == &VirtualKeyCode::LControl
                    || keycode == &VirtualKeyCode::RShift
                    || keycode == &VirtualKeyCode::LShift
                    || keycode == &VirtualKeyCode::RWin
                    || keycode == &VirtualKeyCode::LWin
                {
                    self.modifiers.remove(keycode);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_code(&mut self, event: &WindowEvent) -> bool {
        // shift + enter to play the sound based on self.char_list
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if keycode == &VirtualKeyCode::Return
                    && (self.modifiers.contains(&VirtualKeyCode::LShift)
                        || self.modifiers.contains(&VirtualKeyCode::RShift))
                {
                    let code: String = self.char_list.iter().collect();
                    log::warn!("update code: {}", code);
                    {
                        let mut shared_string_lock = self.shared_string.lock().unwrap();
                        *shared_string_lock = code;
                    }
                    self.has_update
                        .store(true, std::sync::atomic::Ordering::Release);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn update_code(&mut self, event: &WindowEvent) -> bool {
        // shift + enter to play the sound based on self.char_list
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                if keycode == &VirtualKeyCode::Return
                    && (self.modifiers.contains(&VirtualKeyCode::LShift)
                        || self.modifiers.contains(&VirtualKeyCode::RShift))
                {
                    let code: String = self.char_list.iter().collect();
                    log::warn!("update code: {}", code);

                    // use the ringbuf to push
                    // if let Some(engine) = &self.audio_engine {
                    //     let mut engine_borrow = engine.borrow_mut();
                    //     engine_borrow.update_with_code(&code);
                    // }

                    // call the window.run from glicol.js
                    let window = web_sys::window().expect("no global `window` exists");
                    let run = window
                        .get("run")
                        .unwrap()
                        .dyn_into::<js_sys::Function>()
                        .unwrap();
                    let this = JsValue::null();
                    run.call1(&this, &code.into()).unwrap();
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    pub fn move_cursor(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                },
            ..
        } = event
        {
            // tracing::warn!("move cursor left");
            if self.cursors[0] > 0 {
                self.cursors[0] -= 1;
            }
            // tracing::warn!("cursors: {:?}", self.cursors);
            true
        } else if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                },
            ..
        } = event
        {
            // tracing::warn!("move cursor right");
            if self.cursors[0] < self.char_list.len() {
                self.cursors[0] += 1;
            }
            // tracing::warn!("cursors: {:?}", self.cursors);
            true
        } else if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Up),
                    ..
                },
            ..
        } = event
        {
            // tracing::warn!("move cursor up");
            let mut pos = self.cursors[0] as usize;

            // Find the start of the current line
            while pos > 0 && self.char_list[pos - 1] != '\n' {
                pos -= 1;
            }
            let current_line_pos = self.cursors[0] as usize - pos;

            // If we're at the start of the text, we don't move
            if pos == 0 {
                return true;
            }

            let mut new_pos = pos - 2; // Move to the previous character from the start of the current line
            let mut prev_line_start = 0;
            let mut prev_line = 0;

            // Find the start of the previous line
            while new_pos > 0 {
                if self.char_list[new_pos] == '\n' {
                    prev_line += 1;
                    if prev_line == 1 {
                        prev_line_start = new_pos + 1; // After the '\n' character
                        break;
                    }
                }
                new_pos -= 1;
            }

            let target_pos = prev_line_start + current_line_pos;

            // If the target position exceeds the start of the current line, set it to the end of the previous line
            if target_pos >= pos {
                self.cursors[0] = pos - 1; // Just before the current line's start
            } else {
                self.cursors[0] = target_pos;
            }

            true
        } else if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Down),
                    ..
                },
            ..
        } = event
        {
            // tracing::warn!("move cursor down");
            let mut pos = self.cursors[0] as usize;

            // Find the start of the current line
            while pos > 0 && self.char_list[pos - 1] != '\n' {
                pos -= 1;
            }
            let current_line_pos = self.cursors[0] as usize - pos;

            let mut new_pos = self.cursors[0] as usize;
            let mut new_line = 0;
            let mut new_line_pos = 0;

            while new_pos <= self.char_list.len() {
                if new_pos == self.char_list.len() {
                    self.cursors[0] = new_pos;
                    break;
                }
                if self.char_list[new_pos] == '\n' {
                    new_line += 1;
                }
                if new_line == 1 {
                    if new_line_pos == current_line_pos {
                        self.cursors[0] = new_pos + 1;
                        break;
                    }
                    new_line_pos += 1;
                } else if new_line == 2 {
                    self.cursors[0] = new_pos;
                    break;
                }
                new_pos += 1;
            }
            true
        } else {
            false
        }
    }

    pub fn input_or_delete_character(
        &mut self,
        event: &WindowEvent,
        // modifiers: &HashSet<VirtualKeyCode>,
    ) -> bool {
        if let WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Back),
                    ..
                },
            ..
        } = event
        {
            tracing::warn!("delete character");
            if self.cursors[0] >= 1 {
                self.char_list.remove(self.cursors[0] as usize - 1);
                if self.cursors[0] >= 1 {
                    self.cursors[0] -= 1;
                }
            } else {
                // cursor is at the beginning
                if self.char_list.len() > 0 {
                    self.char_list.remove(0);
                }
            }
            true
        } else {
            let c = crate::get_char_from_event(event, &self.modifiers);
            if let Some(c) = c {
                tracing::warn!("add character: {:?}", c);
                self.char_list.insert(self.cursors[0] as usize, c);
                self.cursors[0] += 1;
                true
            } else {
                false
            }
        }
    }

    pub fn update(&mut self) {
        // self.queue.write_buffer(
        //     &self.position_buffer,
        //     0,
        //     bytemuck::cast_slice(&[self.position]),
        // );
        (
            self.render_pipeline,
            self.vertex_buffer,
            self.index_buffer,
            self.num_indices,
            self.diffuse_bind_group,
        ) = crate::utils::update_renderer(
            &self.window,
            &self.device,
            &self.config,
            &self.queue,
            &self.char_list,
            &self.cursors,
            &self.font,
        );
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
