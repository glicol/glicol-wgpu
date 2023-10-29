// self.render_pipeline = render_pipeline;
// self.vertex_buffer = vertex_buffer;
// self.index_buffer = index_buffer;
// self.num_indices = num_indices;
// self.diffuse_bind_group = diffuse_bind_group;

use std::{cell::RefCell, char, rc::Rc};

use fontdue::Font;
use guillotiere::{AtlasAllocator, Size};
use wgpu::{util::DeviceExt, CommandEncoderDescriptor};

use crate::Vertex;

pub fn update_renderer(
    window: &Rc<RefCell<winit::window::Window>>,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    queue: &wgpu::Queue,
    char_list: &Vec<char>,
    font: &Font,
) -> (
    wgpu::RenderPipeline,
    wgpu::Buffer,
    wgpu::Buffer,
    u32,
    wgpu::BindGroup,
) {
    let mut allocator = AtlasAllocator::new(Size::new(2048, 2048));
    let font_size = 32.0 * window.borrow().scale_factor() as f32 * 96. / 72.;
    let padding = 10; // (10. * window.borrow().scale_factor()) as i32;
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
        let font_size_scale = font_size / window.borrow().inner_size().height as f32;

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

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

    (
        render_pipeline,
        vertex_buffer,
        index_buffer,
        num_indices,
        diffuse_bind_group,
    )
}
