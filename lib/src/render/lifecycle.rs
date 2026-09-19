use super::handle;
use std::{pin::Pin, sync::Arc};
use winit::{event_loop as wel, window as ww};

pub enum HostState {
    Resumed,
    Suspended,
}

pub enum GpuState {
    NotStarted,
    Initializing,
    Ready,
    Failed,
}

pub struct Lifecycle {
    host_state: HostState,
    gpu_state: GpuState,
    gpu_handle: Option<handle::Handle>,
}

pub enum LifecycleEffect {
    Initialize(Pin<Box<dyn Future<Output = anyhow::Result<handle::Handle>> + 'static>>),
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            host_state: HostState::Suspended,
            gpu_state: GpuState::NotStarted,
            gpu_handle: None,
        }
    }

    pub fn gpu_handle(&mut self) -> Option<&mut handle::Handle> {
        self.gpu_handle.as_mut()
    }

    pub fn resume(&mut self, event_loop: &wel::ActiveEventLoop) -> Option<LifecycleEffect> {
        let host_state = std::mem::replace(&mut self.host_state, HostState::Suspended);
        let gpu_state = std::mem::replace(&mut self.gpu_state, GpuState::NotStarted);
        match (host_state, gpu_state) {
            (HostState::Suspended, GpuState::NotStarted) => {
                self.gpu_state = GpuState::Initializing;

                #[cfg(target_arch = "wasm32")]
                let window_attributes = {
                    use winit::platform::web::WindowAttributesExtWebSys;
                    ww::Window::default_attributes().with_canvas(Some(options.canvas))
                };

                #[cfg(not(target_arch = "wasm32"))]
                let window_attributes = ww::Window::default_attributes();

                let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
                Some(LifecycleEffect::Initialize(Box::pin(handle::Handle::new(
                    window,
                ))))
            }
            (HostState::Suspended, GpuState::Initializing | GpuState::Ready) => {
                self.host_state = HostState::Resumed;
                None
            }
            (_, GpuState::Failed) => todo!(),
            (HostState::Resumed, _) => None,
        }
    }

    pub fn suspend(&mut self, event_loop: &wel::ActiveEventLoop) {
        todo!();
    }

    pub fn initialize(&mut self, handle: anyhow::Result<handle::Handle>) {
        let gpu_state = std::mem::replace(&mut self.gpu_state, GpuState::NotStarted);
        match gpu_state {
            GpuState::Initializing => match handle {
                Ok(handle) => {
                    self.host_state = HostState::Resumed;
                    self.gpu_handle = Some(handle);
                    let handle = self.gpu_handle.as_mut().unwrap();
                    handle.configure();
                    handle.request_redraw();
                }
                Err(_) => {
                    self.gpu_state = GpuState::Failed;
                }
            },
            _ => panic!("handle expected only in Initializing state"),
        }
    }

    pub fn request_redraw(&self) {
        self.gpu_handle
            .as_ref()
            .map(|handle| handle.request_redraw());
    }
}
