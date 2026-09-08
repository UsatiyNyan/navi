pub(crate) fn spawn_local<F: Future<Output = ()> + 'static>(future: F) {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(future);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::task::spawn_local(future);
    }
}
