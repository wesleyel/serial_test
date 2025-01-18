build-aarch64:
    cross build --target aarch64-unknown-linux-gnu --release

build: build-aarch64

lint:
    cargo fmt
    cargo clippy --all-targets --all-features -- -D warnings