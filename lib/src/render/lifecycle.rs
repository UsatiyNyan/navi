use super::handle::*;

pub enum Lifecycle {
    Init(), // options
    Resumed(Handle),
    Suspended(Handle),
}

impl Lifecycle {
    pub fn new() -> Self {
        Self::Init()
    }
}

// impl winit::ApplicationHandler for Lifecycle { }
