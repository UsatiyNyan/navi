#[cfg(target_arch = "wasm32")]
struct WasmLocalSpawner;

#[cfg(target_arch = "wasm32")]
impl tea::Spawner for WasmLocalSpawner {
    fn spawn(&self, task: tea::Task) {
        wasm_bindgen_futures::spawn_local(task);
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct TokioLocalSpawner;

#[cfg(not(target_arch = "wasm32"))]
impl tea::Spawner for TokioLocalSpawner {
    fn spawn(&self, task: tea::Task) {
        tokio::task::spawn_local(task);
    }
}
