pub(crate) fn spawn_local<F: Future>(future: F) {
    #[cfg(target_arch = "wasm32")]
    {
        todo!();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        todo!();
    }
}
