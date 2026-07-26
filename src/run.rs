use winit::event_loop::EventLoop;
use crate::app::App;


#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(target_arch = "wasm32")]
use console_log::*;
use log;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub fn run() -> anyhow::Result<()> {
    let particle_count = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(4_000_000);
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    {

        let mut app = App::new(particle_count);
        event_loop.run_app(&mut app)?;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let app = App::new(particle_count, &event_loop);
        event_loop.spawn_app(app);
    }

    Ok(())
}