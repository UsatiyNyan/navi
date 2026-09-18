use super::handle;

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

impl Lifecycle {
    pub fn gpu_handle(&mut self) -> Option<&mut handle::Handle> {
        self.gpu_handle.as_mut()
    }
}
