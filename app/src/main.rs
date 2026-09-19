mod app;
mod conductor;
mod platform;

use lib::{render, tea};
use std::rc::Rc;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // TODO: env_logger::init();
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        console_log::init_with_level(log::Level::Info).unwrap_throw();
        console_error_panic_hook::set_once();
    }

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    #[cfg(not(target_arch = "wasm32"))]
    let spawner = Rc::new(platform::TokioSpawner::new());

    #[cfg(target_arch = "wasm32")]
    let spawner = Rc::new(platform::WasmLocalSpawner {});

    let emitter = platform::create_emitter(event_loop_proxy.clone());

    let mut conductor = conductor::Conductor::new(conductor::ConductorSettings {
        app: tea::AppSettings { spawner, emitter },
        event_loop_proxy,
    });

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run_app(&mut conductor).expect("run_app failed");

    #[cfg(target_arch = "wasm32")]
    event_loop.spawn_app(conductor);
}
