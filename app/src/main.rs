use std::rc::Rc;

use lib::tea;

enum Message {
    Decrement,
    Increment,
}

struct AppState {
    app: tea::App<Model, Message>,
}

struct Model {
    counter: i32,
}

impl tea::Model<Message> for Model {
    fn init() -> (Self, tea::Effects<Message>) {
        (Self { counter: 0 }, Default::default())
    }

    fn update(&mut self, message: Message) -> tea::Effects<Message> {
        match message {
            Message::Decrement => self.counter -= 1,
            Message::Increment => self.counter += 1,
        }
        Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "local")]
async fn main() {
    let mut app = tea::App::<Model, Message>::new(tea::AppSettings {
        spawner: Rc::new(TokioLocalSpawner {}),
    });

    let emitter = app.capabilities().emitter().upgrade().unwrap();

    loop {
        // runs effects
        tokio::task::yield_now().await;

        if let Some(model) = app.run_once() {
            println!("counter={}", model.counter);
        }

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("stdin");

        match line.trim().to_lowercase().as_str() {
            "+" => emitter.emit(Message::Increment),
            "-" => emitter.emit(Message::Decrement),
            _ => break,
        }
    }
}

