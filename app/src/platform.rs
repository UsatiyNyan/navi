use crate::{app, conductor, tea};
use winit::event_loop as wel;

#[cfg(target_arch = "wasm32")]
pub(crate) struct WasmLocalSpawner;

#[cfg(target_arch = "wasm32")]
impl tea::Spawner for WasmLocalSpawner {
    fn spawn(&self, task: tea::Task) {
        wasm_bindgen_futures::spawn_local(task);
    }
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
impl tea::Spawner for TokioSpawner {
    fn spawn(&self, task: tea::Task) {
        self.runtime.spawn(task);
    }
}

pub(crate) struct WinitEmitter {
    event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
}

impl tea::Emitter<app::Message> for WinitEmitter {
    fn emit(&self, message: app::Message) {
        let _ = self
            .event_loop_proxy
            .send_event(conductor::ConductorMessage::Message(message));
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn create_emitter(
    event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
) -> tea::EmitterPtr<app::Message> {
    std::rc::Rc::new(WinitEmitter { event_loop_proxy })
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn create_emitter(
    event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
) -> tea::EmitterPtr<app::Message> {
    std::sync::Arc::new(WinitEmitter { event_loop_proxy })
}
