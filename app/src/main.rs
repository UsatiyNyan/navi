mod conductor;
mod app;
mod platform;

use lib::{render, tea};
use std::rc::Rc;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "local")]
async fn main() {
    let spawner = Rc::new(TokioLocalSpawner {});
    let mut tea_app = tea::App::<Model, Message>::new(tea::AppSettings {
        spawner: spawner.clone(),
    });

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let mut render_lifecycle = render::Lifecycle::new(render::LifecycleOptions {
        event_loop_proxy: event_loop.create_proxy(),
        spawner,
    });

    loop {
        // runs effects
        tokio::task::yield_now().await;

        if let Some(model) = tea_app.run_once() {
            render_lifecycle.request_redraw();
        }
    }
}

