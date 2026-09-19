use super::app;
use lib::{buffer, render, tea};
use winit::{application as wa, event as we, event_loop as wel, window as ww};

pub(crate) enum ConductorMessage {
    RenderInitialized(anyhow::Result<render::Handle>),
    MessageAvailable,
    DeviceLost, // TODO
}

pub(crate) struct Conductor {
    tea: tea::App<app::Model, app::Message>,
    render: render::Lifecycle,
    buffer: buffer::State,
    spawner: Rc<dyn tea::Spawner>,
    event_loop_proxy: wel::EventLoopProxy<ConductorMessage>,
}

pub(crate) struct ConductorSettings {
    pub spawner: Rc<dyn tea::Spawner>,
    pub event_loop_proxy: wel::EventLoopProxy<ConductorMessage>,
}

impl Conductor {
    pub(crate) fn new(settings: ConductorSettings) -> Self {
        let tea = tea::App::new(tea::AppSettings {
            spawner: settings.spawner.clone(),
        });
        let render = render::Lifecycle::new();
        let buffer = buffer::State {};
        Self {
            tea,
            render,
            buffer,
            spawner: settings.spawner,
            event_loop_proxy: settings.event_loop_proxy,
        }
    }
}

impl wa::ApplicationHandler<ConductorMessage> for Conductor {
    fn resumed(&mut self, event_loop: &wel::ActiveEventLoop) {
        let lifecycle_action = self.render.resume(event_loop);
        match lifecycle_action {
            Some(render::LifecycleEffect::Initialize(future)) => {
                self.spawner.spawn(Box::pin(async move {
                    self.event_loop_proxy
                        .send_event(ConductorMessage::RenderInitialized(future.await))
                        .expect("send_event failed");
                }));
            }
            None => {}
        }
    }

    fn suspended(&mut self, event_loop: &wel::ActiveEventLoop) {
        self.render.suspend(event_loop);
    }

    fn user_event(&mut self, event_loop: &wel::ActiveEventLoop, event: ConductorMessage) {
        match event {
            ConductorMessage::RenderInitialized(handle) => self.render.initialize(handle),
            ConductorMessage::MessageAvailable => todo!(),
            ConductorMessage::DeviceLost => todo!(),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &wel::ActiveEventLoop,
        _window_id: ww::WindowId,
        event: we::WindowEvent,
    ) {
        match event {
            we::WindowEvent::CloseRequested => event_loop.exit(),
            we::WindowEvent::Resized(size) => self.render.resize(size.width, size.height),
            we::WindowEvent::RedrawRequested => {
                let gpu_handle = self.render.gpu_handle().expect("TODO: invariant");
                app::render(self.tea.model(), gpu_handle, self.buffer)
            }
            _ => {}
        }
    }
}
