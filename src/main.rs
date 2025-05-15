use simul8::*;

async fn run() -> anyhow::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window_builder = winit::window::WindowBuilder::new()
        .with_title("simul8")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

    let window = window_builder.build(&event_loop)?;

    let mut app_state = app::AppState::new(&window, &event_loop).await?;

    app_state.run(event_loop)
}

fn main() {
    env_logger::init();

    let res = pollster::block_on(run());

    if let Err(e) = res {
        util::show_error_dialog(&format!("Fatal error while running simul8: \"{:?}\"", e));
    }
}
