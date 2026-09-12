test:
    cargo test

checks:
    cargo check -p lib
    cargo check -p app
    cargo check -p app --features native-cli

run-cli:
    cargo run -p app --features native-cli --bin app-cli

build-native:
    cargo build -p app --features native-cli --bin app-cli

build-web:
    cargo build -p app --target wasm32-unknown-unknown
