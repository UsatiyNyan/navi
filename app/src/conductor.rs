use super::{app, visualization};
use lib::{buffer, render, tea};
use std::rc::Rc;
use winit::{application as wa, event as we, event_loop as wel, window as ww};

#[derive(Debug)]
pub(crate) enum ConductorMessage {
    RenderInitialized(anyhow::Result<render::Handle>),
    Message(app::Message),
}

pub(crate) struct Conductor {
    tea: tea::App<app::Model, app::Message>,
    render: render::Lifecycle,
    buffer: buffer::State<app::Record>,
    spawner: Rc<dyn tea::Spawner>,
    event_loop_proxy: wel::EventLoopProxy<ConductorMessage>,
    visualization: Option<visualization::Visualization>,
}

pub(crate) struct ConductorSettings {
    pub app: tea::AppSettings<app::Message>,
    pub event_loop_proxy: wel::EventLoopProxy<ConductorMessage>,
    pub window_attributes: ww::WindowAttributes,
}

impl Conductor {
    pub(crate) fn new(settings: ConductorSettings) -> Self {
        let spawner = settings.app.spawner.clone();
        let tea = tea::App::new(settings.app);
        let render = render::Lifecycle::new(settings.window_attributes);
        let buffer = buffer::State::new(buffer::TimelineOptions {
            window: buffer::Duration::from_secs(120),
        });
        Self {
            tea,
            render,
            buffer,
            spawner,
            event_loop_proxy: settings.event_loop_proxy,
            visualization: None,
        }
    }
}

impl wa::ApplicationHandler<ConductorMessage> for Conductor {
    fn resumed(&mut self, event_loop: &wel::ActiveEventLoop) {
        let lifecycle_action = self.render.resume(event_loop);
        match lifecycle_action {
            Some(render::LifecycleEffect::Initialize(future)) => {
                let event_loop_proxy = self.event_loop_proxy.clone();
                self.spawner.spawn(Box::pin(async move {
                    let _ = event_loop_proxy
                        .send_event(ConductorMessage::RenderInitialized(future.await));
                }));
            }
            None => {}
        }
    }

    fn suspended(&mut self, event_loop: &wel::ActiveEventLoop) {
        self.render.suspend(event_loop);
    }

    fn user_event(&mut self, _event_loop: &wel::ActiveEventLoop, event: ConductorMessage) {
        match event {
            ConductorMessage::RenderInitialized(handle) => {
                self.render.initialize(handle);
                if let Some(gpu_handle) = self.render.gpu_handle() {
                    self.visualization = Some(visualization::Visualization::new(gpu_handle));
                }
            }
            ConductorMessage::Message(message) => self.tea.enqueue(message),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &wel::ActiveEventLoop,
        _window_id: ww::WindowId,
        event: we::WindowEvent,
    ) {
        log::debug!("Conductor::window_event {:?}", event);

        match event {
            we::WindowEvent::CloseRequested => event_loop.exit(),
            we::WindowEvent::Resized(size) => self.render.resize(size),
            we::WindowEvent::RedrawRequested => {
                if let (Some(gpu_handle), Some(visualization)) =
                    (self.render.gpu_handle(), &self.visualization)
                {
                    let _ = visualization
                        .render(self.tea.model(), gpu_handle, &mut self.buffer)
                        .map_err(|err| log::error!("{err}"));
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &wel::ActiveEventLoop) {
        if self.tea.run_once().is_some() {
            self.render.request_redraw();
        }
    }
}
