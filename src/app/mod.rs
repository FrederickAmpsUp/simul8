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

    timeline_pos: f32,
    timeline_range: std::ops::RangeInclusive<f32>,

    pub sim_state: crate::sim::SimulationState<'a>,
    sim_renderer: Box<dyn crate::sim::rendering::SimRenderer>,

    window: &'a winit::window::Window
}

impl<'a> AppState<'a> {
    pub async fn new(window: &'a winit::window::Window) -> anyhow::Result<Self> {
        log::info!("App initialization");

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::default(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::from_env_or_default()
        });

        log::info!(" - WGPU instance acquired.");

        let window_surface = instance.create_surface(window)?;

        log::info!(" - Window surface created.");

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&window_surface),
            force_fallback_adapter: false
        }).await.ok_or(anyhow::anyhow!("Failed to get adapter"))?;

        log::info!(" - Acquired adapter \"{}\"", adapter.get_info().name);

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            required_features: wgpu::Features::empty(),

            label: None,
            memory_hints: Default::default()
        }, None).await?;

        log::info!(" - Acquired device and queue.");

        let window_surface_caps = window_surface.get_capabilities(&adapter);

        let window_surface_format = window_surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(window_surface_caps.formats[0]);

        if !window_surface_format.is_srgb() {
            log::warn!("No surface format found supporting sRGB!");
        }

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

        log::info!(" - Created EGUI objects.");

        let mut sim_state = crate::sim::SimulationState::new();
        sim_state.gravity_accel = glam::vec2(0.0, 0.1);

        let sim_renderer = Box::new(crate::sim::rendering::CpuSimRenderer::new());

        Ok(Self {
            window_surface,
            window_surface_config, window_size,

            needs_reconfigure: true,

            device, queue,

            egui_state, egui_renderer,

            timeline_pos: 0.0,
            timeline_range: 0.0..=3.5,

            sim_state, sim_renderer,

            window
        })
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            log::info!("Resize to {}x{} requested.", new_size.width, new_size.height);
            self.window_size = new_size;
            self.needs_reconfigure = true;
        }
    }

    pub fn reconfigure(&mut self) {
        self.needs_reconfigure = true;
    }

    pub fn build_ui(&mut self, egui_input: egui::RawInput) -> egui::FullOutput {
        let preview_aspect = 9.0 / 16.0;

        self.egui_state.egui_ctx().run(egui_input, |ctx| {
            egui::SidePanel::left("tools_panel").resizable(true).show(ctx, |ui| {
                ui.heading("Tools");

                ui.horizontal_wrapped(|ui| {
                    ui.label("Lorem ipsum dolor sit amet");
                });
            });

            let available_height = ctx.available_rect().height();

            let timeline_max_ratio = 1.0 / 5.0;
            let timeline_min_ratio = (3.0/8.0) / 5.0;

            let timeline_max_height = available_height * timeline_max_ratio;
            let timeline_min_height = available_height * timeline_min_ratio;

            egui::TopBottomPanel::bottom("timeline_panel")
                .resizable(true)
                .max_height(timeline_max_height).min_height(timeline_min_height)
                .show(ctx, |ui| {
                let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

                let slider_left = rect.left() + rect.width()*0.05;
                let slider_right = rect.right() - rect.width()*0.05;

                let playhead_width = rect.width() * 0.01;
                let playhead_height = rect.height() * 0.9;

                let slider_top = rect.top();
                let slider_bottom = rect.bottom();

                let slider_cy = rect.top() + rect.height()*0.5;

                let tick_major_height = playhead_height / 2.0;
                let tick_minor_height = tick_major_height / 2.0;

                let painter = ui.painter();

                if response.dragged() || response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if rect.contains(pos) || rect.contains(ui.input(|i| i.pointer.press_origin().unwrap_or(egui::pos2(rect.left()-1.0, rect.top()-1.0)))) {
                            self.timeline_pos = egui::remap_clamp(pos.x, slider_left..=slider_right, self.timeline_range.clone());
                        }
                    }
                }

                painter.line_segment([egui::pos2(slider_left, slider_cy), egui::pos2(slider_right, slider_cy)], egui::Stroke::new(1.0, egui::Color32::GRAY));

                let tick_count = ((self.timeline_range.end() - self.timeline_range.start()) * 4.0).floor() as usize;

                for i in 0..=tick_count {
                    let v = self.timeline_range.start() + (i as f32 * 0.25);

                    let tick_pos = egui::remap(v, self.timeline_range.clone(), slider_left..=slider_right);
                    let h = if i % 4 == 0 { tick_major_height } else { tick_minor_height } / 2.0;

                    if i % 4 == 0 {
                        painter.text(egui::pos2(tick_pos, slider_cy + h*1.5), egui::Align2::CENTER_CENTER, v.to_string(), egui::FontId::default(), egui::Color32::GRAY);
                    }

                    painter.line_segment([egui::pos2(tick_pos, slider_cy+h), egui::pos2(tick_pos, slider_cy-h)], egui::Stroke::new(1.0, egui::Color32::GRAY));
                }

                let playhead_pos = egui::remap(self.timeline_pos, self.timeline_range.clone(), slider_left..=slider_right);

                painter.rect_filled(egui::Rect::from_center_size(egui::pos2(playhead_pos, slider_cy), egui::vec2(playhead_width, playhead_height)), playhead_width/3.0, egui::Color32::WHITE);

            });

            egui::CentralPanel::default().show(ctx, |ui| {
                let available_size = ui.available_size();

                let preview_height = available_size.y;
                let preview_width = preview_height * preview_aspect;

                egui::SidePanel::right("preview_panel")
                    .exact_width(preview_width)
                    .show_inside(ui, |ui| {
                    self.sim_renderer.render(&self.sim_state, ui);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    ui.heading("Other Stuff...");
                });
            });
        })
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.sim_state.solve_pbd(1.0/60.0);

        if self.needs_reconfigure {
            self.needs_reconfigure = false;
            self.window_surface_config.width = self.window_size.width;
            self.window_surface_config.height = self.window_size.height;

            self.window_surface.configure(&self.device, &self.window_surface_config);

            log::info!("Reconfigured window surface (size {}x{})", self.window_surface_config.width, self.window_surface_config.height);
        }

        let ppp = self.window.scale_factor() as f32;

        self.egui_state.egui_ctx().set_pixels_per_point(ppp);
        //self.egui_state.egui_ctx().set_debug_on_hover(true);

        let egui_input = self.egui_state.take_egui_input(&self.window);

        let egui_output = self.build_ui(egui_input);

        self.egui_state.handle_platform_output(&self.window, egui_output.platform_output);
        let paint_jobs = self.egui_state.egui_ctx().tessellate(egui_output.shapes, ppp);

        let window_surface_texure = self.window_surface.get_current_texture()?;

        let window_surface_view = window_surface_texure.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder")
        });

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.window_size.width, self.window_size.height],
            pixels_per_point: ppp
        };

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_desc
        );

        for (id, image_delta) in &egui_output.textures_delta.set {
            self.egui_renderer.update_texture(
                &self.device,
                &self.queue,
                *id,
                image_delta
            );
        }

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            let mut render_pass = render_pass.forget_lifetime();

            self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_desc);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        window_surface_texure.present();

        for id in &egui_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

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
