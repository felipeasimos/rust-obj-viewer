use wgpu::util::DeviceExt;
use winit::window::Window;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, WindowEvent};

use crate::camera;

pub struct Engine {

    // create surface and adapter
    instance: wgpu::Instance,
    // open connection to GPU, creating device
    adapter: wgpu::Adapter,
    // used to interact with the GPU
    device: wgpu::Device,
    // holds the texture we will write to
    surface: wgpu::Surface,
    // used to prepare surfaces for presentation
    surface_config: wgpu::SurfaceConfiguration,
    // used to write to buffers and texture by executing recorded commands
    queue: wgpu::Queue,
    // render pipeline being used
    render_pipeline: wgpu::RenderPipeline,
    // screen size
    window_size: winit::dpi::PhysicalSize<u32>,
    // camera
    camera: camera::CameraData,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    mouse_pressed: bool
}

impl Engine {

    pub async fn new(window: &Window) -> Self {

        let window_size = window.inner_size();
        let instance = Engine::create_instance();
        let surface = Engine::create_surface(&instance, window);
        let adapter = Engine::request_adapter(&instance, &surface).await;
        let (device, queue) = Engine::request_device_and_queue(&adapter).await;
        let surface_config = Engine::create_surface_config(&adapter, &surface, &window_size);
        surface.configure(&device, &surface_config);

        let camera = camera::CameraData::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let camera_controller = camera::CameraController::new(0.1, 0.1);
        let projection = camera::Projection::new(surface_config.width, surface_config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ],
            label: Some("camera_bind_group_layout")
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {

            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
            label: Some("camera_bind_group")
        });

        let render_pipeline = Engine::create_render_pipeline(&device, &surface_config, &camera_bind_group_layout);
        Self {
            instance,
            adapter,
            device,
            surface,
            surface_config,
            queue,
            render_pipeline,
            window_size,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            mouse_pressed: false
        }
    }

    fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(wgpu::Backends::all())
    }
    fn create_surface(instance: &wgpu::Instance, window: &Window) -> wgpu::Surface {
        unsafe { instance.create_surface(window) }
    }
    async fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(surface),
                force_fallback_adapter: false
            }
        ).await.unwrap()
    }
    async fn request_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: Some("Engine Device")
            },
            None
        ).await.unwrap()
    }
    fn create_surface_config(adapter: &wgpu::Adapter, surface: &wgpu::Surface, window_size: &winit::dpi::PhysicalSize<u32>) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo
        }
    }
    fn create_render_pipeline(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration, camera_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::RenderPipeline {

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
        });
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                camera_bind_group_layout
            ],
            push_constant_ranges: &[]
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                }]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            }
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height);
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(
                KeyboardInput {
                    virtual_keycode: Some(key),
                    state,
                    ..
                }
            ) => self.camera_controller.process_keyboard(*key, *state),
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(&delta);
                true
            }
            DeviceEvent::Button {
                button: 1,
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size.clone()
    }
}
