// TODOs/Nice to haves
// rename shader_uniform to uniform and wrap mouse_position, resolution, zoom, etc. in a struct
// tweak mouse position to be in the range of the fractal (eg. -2.0 to 2.0)
// change sensitivity of mouse based on zoom level
// or maybe consider switching to WASD controls
// look at restructuring the code to be more modular
// add egui
// write a struct for mandelbrot/fractal parameters
// let these be changed by egui
mod ui;

use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Mouse {
    //            align(16) size(32)
    x: f32,        // offset(0)  align(4)  size(4)
    y: f32,        // offset(4)  align(4)  size(4)
    drag: i32,     // offset(8)  align(4)  size(4)
    px: f32,       // offset(12) align(4)  size(4)
    py: f32,       // offset(16) align(4)  size(4)
    centre_x: f32, // offset(20) align(4)  size(4)
    centre_y: f32, // offset(24) align(4)  size(4)
    _pad: u32,     // offset(28)           size(4)
}

impl Mouse {
    fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            drag: -1,
            px: 0.0,
            py: 0.0,
            centre_x: -0.765,
            centre_y: 0.0,
            _pad: 0,
        }
    }

    fn set_drag(&mut self) {
        self.drag = 1;
    }

    fn unset_drag(&mut self) {
        self.drag = 0;
    }

    fn update_position(&mut self, x: f32, y: f32) {
        self.px = self.x;
        self.py = self.y;
        self.x = -x;
        self.y = -y;
    }

    fn drag_mouse(&mut self, resolution: [f32; 2], axis_range: f32) {
        if self.drag == 1 {
            let ratio = resolution[0] / resolution[1];
            let x = self.x - self.px;
            let y = self.y - self.py;
            let x = axis_range * x / resolution[0] * ratio;
            let y = axis_range * y / resolution[1];
            self.centre_x += x;
            self.centre_y += y;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniform {
    //            align(16) size(64)
    resolution: [f32; 2], // offset(0)  align(8)  size(8)
    iterations: i32,      // offset(8)  align(4)  size(4)
    value: f32,           // offset(12) align(4)  size(4)
    mouse: Mouse,         // offset(16) align(16) size(32)
    axis_range: f32,      // offset(48) align(4)  size(4)
    exponent: f32,        // offset(48) align(4)  size(4)
    _pad: [u32; 2],
}

impl ShaderUniform {
    fn new() -> Self {
        Self {
            resolution: [0.0, 0.0],
            iterations: 100,
            value: 2.0,
            mouse: Mouse::new(),
            axis_range: 2.0,
            exponent: 2.0,
            _pad: [0; 2],
        }
    }

    fn update_resolution(&mut self, width: f32, height: f32) {
        self.resolution = [width, height];
    }

    fn update_iterations(&mut self, iterations: i32) {
        self.iterations = iterations;
    }

    fn update_value(&mut self, value: f32) {
        self.value = value;
    }

    fn update_exponent(&mut self, exponent: f32) {
        self.exponent = exponent;
    }
}
struct UiWrapper {
    ctx: egui::Context,
    wgpu_ctx: egui_wgpu::Renderer,
    winit_ctx: egui_winit::State,
    interface: ui::Interface,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    colour: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, 0.0],
        colour: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0],
        colour: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        colour: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        colour: [0.3, 0.4, 0.3],
    },
];

const INDICES: &[u16] = &[0, 1, 3, 1, 2, 3];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    shader_uniform: ShaderUniform,
    shader_buffer: wgpu::Buffer,
    shader_bind_group: wgpu::BindGroup,
    ui_wrapper: UiWrapper,
}

impl State {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut shader_uniform = ShaderUniform::new();
        shader_uniform.update_resolution(
            window.inner_size().width as f32,
            window.inner_size().height as f32,
        );

        let shader_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shader uniform"),
            contents: bytemuck::cast_slice(&[shader_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shader_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("shader_bind_group_layout"),
            });

        let shader_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shader_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shader_buffer.as_entire_binding(),
            }],
            label: Some("shader_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&shader_bind_group_layout],
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let egui_ctx = egui::Context::default();
        let wgpu_ctx = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
        let winit_ctx = egui_winit::State::new(&window);

        let ui_wrapper = UiWrapper {
            ctx: egui_ctx,
            wgpu_ctx,
            winit_ctx,
            interface: ui::Interface::new(),
        };

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            shader_uniform,
            shader_buffer,
            shader_bind_group,
            ui_wrapper,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.shader_uniform
                .update_resolution(new_size.width as f32, new_size.height as f32);
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.ui_wrapper
            .winit_ctx
            .on_event(&self.ui_wrapper.ctx, event)
            .consumed;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.shader_uniform
                    .mouse
                    .update_position(position.x as f32, position.y as f32);
                self.shader_uniform.mouse.drag_mouse(
                    self.shader_uniform.resolution,
                    self.shader_uniform.axis_range,
                );
                true
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
                ..
            } => {
                self.shader_uniform.mouse.set_drag();
                true
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: ElementState::Released,
                ..
            } => {
                self.shader_uniform.mouse.unset_drag();
                true
            }

            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => {
                    if *y < 0.0 {
                        self.shader_uniform.axis_range *= 1.05
                    } else {
                        self.shader_uniform.axis_range *= 0.95
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self) {
        if self.shader_uniform.iterations != self.ui_wrapper.interface.iterations {
            self.shader_uniform
                .update_iterations(self.ui_wrapper.interface.iterations);
        }

        if self.shader_uniform.value != self.ui_wrapper.interface.value {
            self.shader_uniform
                .update_value(self.ui_wrapper.interface.value)
        }

        if self.shader_uniform.exponent != self.ui_wrapper.interface.exponent {
            self.shader_uniform
                .update_exponent(self.ui_wrapper.interface.exponent);
        }
        self.queue.write_buffer(
            &self.shader_buffer,
            0,
            bytemuck::cast_slice(&[self.shader_uniform]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut command_buffer = Vec::new();

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
            render_pass.set_bind_group(0, &self.shader_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // egui pass
        {
            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [self.size.width as u32, self.size.height as u32],
                pixels_per_point: self.ui_wrapper.winit_ctx.pixels_per_point(),
            };

            let input = self.ui_wrapper.winit_ctx.take_egui_input(&self.window);
            let output = self.ui_wrapper.ctx.run(input, |ctx| {
                self.ui_wrapper.interface.ui(ctx);
            });

            self.ui_wrapper.winit_ctx.handle_platform_output(
                &self.window,
                &self.ui_wrapper.ctx,
                output.platform_output,
            );

            let texture_deltas = output.textures_delta;
            let paint_jobs = self.ui_wrapper.ctx.tessellate(output.shapes);

            for (id, image_delta) in &texture_deltas.set {
                self.ui_wrapper.wgpu_ctx.update_texture(
                    &self.device,
                    &self.queue,
                    *id,
                    image_delta,
                );
            }

            for id in &texture_deltas.free {
                self.ui_wrapper.wgpu_ctx.free_texture(id);
            }

            let ui_commands = self.ui_wrapper.wgpu_ctx.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.ui_wrapper
                .wgpu_ctx
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);

            command_buffer.extend(ui_commands.into_iter());
        }

        self.queue.submit(
            command_buffer
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );

        output.present();

        Ok(())
    }
}

pub async fn run() {
    // Setup logging
    env_logger::init();

    // Window Setup
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("mandelbrot");

    // State
    let mut state = State::new(window).await;

    // Event Loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => {}
    });
}
