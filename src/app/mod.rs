use winit::event::{Event, WindowEvent};

#[allow(dead_code)]
pub struct AppState<'a> {
    window_surface: wgpu::Surface<'a>,
    window_surface_config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    needs_reconfigure: bool,

    device: wgpu::Device,
    queue: wgpu::Queue,

    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    window: &'a winit::window::Window
}

impl<'a> AppState<'a> {
    pub async fn new(window: &'a winit::window::Window, event_loop: &winit::event_loop::EventLoop<()>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::default(), // possibly add web support in the future
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default()
        });

        let window_surface = instance.create_surface(window)?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&window_surface),
            force_fallback_adapter: false
        }).await.ok_or(anyhow::anyhow!("Failed to get adapter"))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::default(),
            required_features: wgpu::Features::empty(),

            label: None,
            memory_hints: Default::default()
        }, None).await?;

        let window_surface_caps = window_surface.get_capabilities(&adapter);

        let window_surface_format = window_surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(window_surface_caps.formats[0]);

        let window_size = window.inner_size();

        let window_surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: window_surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: window_surface_caps.present_modes[0],
            alpha_mode: window_surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx,
            egui::ViewportId::ROOT,
            window,
            Some(1.0),
            Some(winit::window::Theme::Dark),
            Some(2048)
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            window_surface_config.format,
            None,
            1, false
        );

        Ok(Self {
            window_surface,
            window_surface_config, window_size,

            needs_reconfigure: true,

            device, queue,

            egui_state, egui_renderer,

            window
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.needs_reconfigure = true;
        }
    }

    pub fn reconfigure(&mut self) {
        self.needs_reconfigure = true;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.needs_reconfigure {
            self.needs_reconfigure = false;
            self.window_surface_config.width = self.window_size.width;
            self.window_surface_config.height = self.window_size.height;

            self.window_surface.configure(&self.device, &self.window_surface_config);
        }

        let window_surface_texure = self.window_surface.get_current_texture()?;

        let window_surface_view = window_surface_texure.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder")
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &window_surface_view,
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
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        window_surface_texure.present();

        Ok(())
    }

    pub fn window(&self) -> &winit::window::Window {
        self.window
    }

    pub fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) -> anyhow::Result<()> {
        let mut exit_status: anyhow::Result<()> = Ok(());
        let exit = &mut exit_status;

        #[allow(deprecated)]
        event_loop.run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id
                } if window_id == self.window.id() => {
                    if self.egui_state.on_window_event(&self.window, event).consumed {
                        return;
                    }

                    match event {
                        WindowEvent::CloseRequested => control_flow.exit(),

                        WindowEvent::RedrawRequested => {
                            match self.render() {
                                Ok(_) => {},
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => self.reconfigure(),
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    *exit = Err(anyhow::anyhow!("Ran out of GPU memory!"));
                                    control_flow.exit();
                                },
                                Err(wgpu::SurfaceError::Other) => {
                                    *exit = Err(anyhow::anyhow!("Unknown rendering error!"));
                                    control_flow.exit();
                                },
                                _ => {}
                            }
                            self.window().request_redraw();
                        },

                        WindowEvent::Resized(new_size) => self.resize(*new_size),
                        _ => {}
                    }
                },
                _ => {}
            }})?;

        exit_status
    }
}
