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
    Ready(handle::Handle),
    Failed,
}

pub struct Lifecycle {
    host_state: HostState,
    gpu_state: GpuState,
}

#[cfg(not(target_arch = "wasm32"))]
type InitializationFuture =
    Pin<Box<dyn Future<Output = anyhow::Result<handle::Handle>> + Send + 'static>>;

#[cfg(target_arch = "wasm32")]
type InitializationFuture = Pin<Box<dyn Future<Output = anyhow::Result<handle::Handle>> + 'static>>;

pub enum LifecycleEffect {
    Initialize(InitializationFuture),
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            host_state: HostState::Suspended,
            gpu_state: GpuState::NotStarted,
        }
    }

    pub fn gpu_handle(&mut self) -> Option<&mut handle::Handle> {
        match &mut self.gpu_state {
            GpuState::Ready(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn resume(&mut self, event_loop: &wel::ActiveEventLoop) -> Option<LifecycleEffect> {
        match (&self.host_state, &self.gpu_state) {
            (HostState::Suspended, GpuState::NotStarted) => {
                self.host_state = HostState::Resumed;
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
            (HostState::Suspended, GpuState::Initializing | GpuState::Ready(_)) => {
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
                Ok(mut handle) => {
                    handle.configure();
                    handle.request_redraw();
                    self.gpu_state = GpuState::Ready(handle);
                }
                Err(_) => {
                    self.gpu_state = GpuState::Failed;
                }
            },
            _ => panic!("handle expected only in Initializing state"),
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if let GpuState::Ready(handle) = &mut self.gpu_state {
            handle.resize(size.width, size.height);
        }
    }

    pub fn request_redraw(&self) {
        if let GpuState::Ready(handle) = &self.gpu_state {
            handle.request_redraw();
        }
    }
}
