use super::capabilities::Capabilities;
use std::future::Future;

pub type Effects<Message> = Vec<Effect<Message>>;
pub type Effect<Message> = Box<dyn FnOnce(&Capabilities<Message>)>; // FOR LATER: don't like Box<dyn> here

pub fn from_future<Message: 'static, F: Future<Output = Message> + 'static>(
    future: F,
) -> Effect<Message> {
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
