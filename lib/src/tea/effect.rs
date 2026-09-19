use super::capabilities::Capabilities;
use std::future::Future;

pub type Effects<Message> = Vec<Effect<Message>>;
pub type Effect<Message> = Box<dyn FnOnce(&Capabilities<Message>)>; // FOR LATER: don't like Box<dyn> here

#[cfg(target_arch = "wasm32")]
pub fn from_future<Message, F>(future: F) -> Effect<Message>
where
    Message: 'static,
    F: Future<Output = Message> + 'static,
{
    Box::new(move |capabilities| {
        let capability_emit = capabilities.emitter();
        capabilities.spawn(Box::pin(async move {
            let message = future.await;
            if let Some(emitter) = capability_emit.upgrade() {
                emitter.emit(message);
            }
        }));
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn from_future<Message, F>(future: F) -> Effect<Message>
where
    Message: Send + 'static,
    F: Future<Output = Message> + Send + 'static,
{
    Box::new(move |capabilities| {
        let capability_emit = capabilities.emitter();
        capabilities.spawn(Box::pin(async move {
            let message = future.await;
            if let Some(emitter) = capability_emit.upgrade() {
                emitter.emit(message);
            }
        }));
    })
}
