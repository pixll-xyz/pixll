use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use web_sys::{HtmlCanvasElement, window};
use web_sys::MouseEvent;
use log::info;

#[derive(Clone, Copy)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

pub enum Component {
    Button { rect: Rect, text: String, on_click: Option<fn() -> ()> },
    Slider { rect: Rect, value: f32, min: f32, max: f32, on_change: Option<fn(f32) -> ()> },
    Text { rect: Rect, content: String },
    Image { rect: Rect, texture: wgpu::Texture },
}

pub struct Pixll {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    components: Vec<Component>,
    mouse_pos: (f32, f32),
    layout: Vec<(Component, Rect)>,
}

impl Pixll {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        console_log::init_with_level(log::Level::Info).unwrap();
        console_error_panic_hook::set_once();

        // Set up WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface_from_canvas(&canvas)?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or("Failed to find an appropriate adapter")?;

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                label: None,
            },
            None,
        ).await.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas.width(),
            height: canvas.height(),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create a simple render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create a vertex buffer for a simple rectangle (test rendering)
        let vertices: &[f32] = &[
            // Position (x, y), Color (r, g)
            -0.5, -0.5, 1.0, 0.0,  // Bottom-left
             0.5, -0.5, 0.0, 1.0,  // Bottom-right
             0.0,  0.5, 0.0, 0.0,  // Top
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            vertex_buffer,
            components: Vec::new(),
        })
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.push(component);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn handle_mouse_click(&mut self, event: MouseEvent) {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        self.mouse_pos = (x, y);

        for component in &mut self.components {
            match component {
                Component::Button { rect, on_click, .. } => {
                    if x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height {
                        if let Some(callback) = on_click {
                            callback();
                        }
                    }
                }
                Component::Slider { rect, value, min, max, on_change, .. } => {
                    if x >= rect.x && x <= rect.x + rect.width && y >= rect.y && y <= rect.y + rect.height {
                        let t = (x - rect.x) / rect.width;
                        *value = min + t * (max - min);
                        if let Some(callback) = on_change {
                            callback(*value);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn layout_vertical(&mut self, components: Vec<Component>, start_x: f32, start_y: f32, spacing: f32) {
        let mut y = start_y;
        self.layout.clear();
        for mut component in components {
            let rect = match &mut component {
                Component::Button { rect, .. } => rect,
                Component::Slider { rect, .. } => rect,
                Component::Text { rect, .. } => rect,
                Component::Image { rect, .. } => rect,
            };
            rect.x = start_x;
            rect.y = y;
            self.layout.push((component.clone(), *rect));
            y += rect.height + spacing;
        }
        self.components = self.layout.iter().map(|(comp, _)| comp.clone()).collect();
    }
}