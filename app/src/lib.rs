//! Web host-facing app surface (planned).
//!
//! TODO: expose a stable `AppState` API for wasm host integration:
//! - create/init state
//! - send input messages
//! - run one tick (`run_once`)
//! - read view/model snapshot

#[cfg(target_arch = "wasm32")]
struct WasmLocalSpawner;

#[cfg(target_arch = "wasm32")]
impl tea::Spawner for WasmLocalSpawner {
    fn spawn(&self, task: tea::Task) {
        wasm_bindgen_futures::spawn_local(task);
    }
}
