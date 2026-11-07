use super::capabilities::Capabilities;
use std::future::Future;

pub type Effects<Message, Ext> = Vec<Effect<Message, Ext>>;
pub type Effect<Message, Ext> = Box<dyn FnOnce(&Capabilities<Message, Ext>)>; // FOR LATER: don't like Box<dyn> here

#[cfg(target_arch = "wasm32")]
pub fn from_future<Message, Ext, F>(future: F) -> Effect<Message, Ext>
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
pub fn from_future<Message, Ext, F>(future: F) -> Effect<Message, Ext>
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
