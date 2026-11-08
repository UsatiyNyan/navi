use crate::{app, conductor};
use winit::event_loop as wel;

pub(crate) struct WinitEmitter {
    event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
}

impl WinitEmitter {
    pub(crate) fn new(
        event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
    ) -> lib::tea::EmitterPtr<app::Message> {
        #[cfg(target_arch = "wasm32")]
        return std::rc::Rc::new(WinitEmitter { event_loop_proxy });

        #[cfg(not(target_arch = "wasm32"))]
        return std::sync::Arc::new(WinitEmitter { event_loop_proxy });
    }
}

impl lib::tea::Emitter<app::Message> for WinitEmitter {
    fn emit(&self, message: app::Message) {
        let _ = self
            .event_loop_proxy
            .send_event(conductor::ConductorMessage::Message(message));
    }
}

pub(crate) struct InjestExt {
    event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
}

impl InjestExt {
    pub(crate) fn new(
        event_loop_proxy: wel::EventLoopProxy<conductor::ConductorMessage>,
    ) -> lib::tea::ExtPtr<Self> {
        lib::tea::ExtPtr::new(InjestExt { event_loop_proxy })
    }

    pub fn injest(&self, payload: app::BufferPayload) {
        let _ = self
            .event_loop_proxy
            .send_event(conductor::ConductorMessage::Injest(payload));
    }
}
