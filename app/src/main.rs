mod app;
mod conductor;
mod platform;
mod visualization;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let event_loop_proxy = event_loop.create_proxy();

    let spawner = std::rc::Rc::new(TokioSpawner::new());
    let emitter = platform::WinitEmitter::new(event_loop_proxy.clone());

    let mut conductor = conductor::Conductor::new(conductor::ConductorSettings {
        app: lib::tea::AppSettings { spawner, emitter },
        event_loop_proxy,
        window_attributes: winit::window::Window::default_attributes(),
    });

    event_loop.run_app(&mut conductor).expect("run_app failed");
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct TokioSpawner {
    runtime: tokio::runtime::Runtime,
}

#[cfg(not(target_arch = "wasm32"))]
impl TokioSpawner {
    pub(crate) fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        Self { runtime }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl lib::tea::Spawner for TokioSpawner {
    fn spawn(&self, task: lib::tea::Task) {
        self.runtime.spawn(task);
    }
}
