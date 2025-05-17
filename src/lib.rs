pub mod app;
pub mod util;
pub mod sim;

use anyhow::Context;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
pub fn setup() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger!");

    log::info!("Setup logging & panic system");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup() {
    env_logger::init();
}

#[cfg(target_arch = "wasm32")]
pub fn configure_window_postcreate(window: winit::window::Window) -> anyhow::Result<winit::window::Window> {
    use winit::platform::web::WindowExtWebSys;

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let window_canvas_container = doc.get_element_by_id("window-canvas-container")?;
            let canvas = web_sys::Element::from(window.canvas()?);
        
            window_canvas_container.append_child(&canvas).ok()?;
            Some(())
        }).context("Failed to append canvas to document body.")?;

    Ok(window)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn configure_window_postcreate(window: winit::window::Window) -> anyhow::Result<winit::window::Window> {
    Ok(window)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    if let Err(e) = run().await {
        util::show_error_dialog(&format!("Fatal error while running simul8: \"{:?}\"", e));
    }
}

pub async fn run() -> anyhow::Result<()> {
    setup();

    let event_loop = winit::event_loop::EventLoop::new()?;

    #[allow(deprecated)]
    let window = event_loop.create_window(winit::window::WindowAttributes::default()
        .with_title("simul8")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600))
    )?;

    let window = configure_window_postcreate(window)?;

    let mut app_state = app::AppState::new(&window).await?;

    app_state.sim_state.as_mut().expect("").add_particle(sim::Particle {
        position: glam::Vec2::ZERO,
        last_position: glam::Vec2::ZERO,
    
        radius: 0.1,
        color: egui::Color32::RED
    });

    app_state.run(event_loop)
}
