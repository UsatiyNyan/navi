mod app;
mod conductor;
mod platform;
mod visualization;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas: web_sys::HtmlCanvasElement) {
    console_log::init_with_level(log::Level::Info).unwrap_throw();
    console_error_panic_hook::set_once();

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    let spawner = std::rc::Rc::new(WasmLocalSpawner {});
    let emitter = platform::WinitEmitter::new(event_loop_proxy.clone());

    use winit::platform::web::WindowAttributesExtWebSys;
    let conductor = conductor::Conductor::new(conductor::ConductorSettings {
        app: lib::tea::AppSettings { spawner, emitter },
        event_loop_proxy,
        window_attributes: winit::window::Window::default_attributes().with_canvas(Some(canvas)),
    });

    use winit::platform::web::EventLoopExtWebSys;
    event_loop.spawn_app(conductor);
}

#[cfg(target_arch = "wasm32")]
pub(crate) struct WasmLocalSpawner;

#[cfg(target_arch = "wasm32")]
impl lib::tea::Spawner for WasmLocalSpawner {
    fn spawn(&self, task: lib::tea::Task) {
        wasm_bindgen_futures::spawn_local(task);
    }
}
