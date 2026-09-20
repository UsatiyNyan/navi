use crate::tea;

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
