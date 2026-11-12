test:
    cargo test --workspace --features app/native-cli

checks:
    cargo clippy --workspace --all-targets --features app/native-cli -- -D warnings
    cargo clippy -p app --lib --target wasm32-unknown-unknown -- -D warnings

run-cli:
    cargo run -p app --features native-cli --bin app-cli

build-native:
    cargo build -p app --features native-cli --bin app-cli

build-web:
    cargo build -p app --target wasm32-unknown-unknown
