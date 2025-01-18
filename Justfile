build-aarch64:
    cross build --target aarch64-unknown-linux-gnu --release

build: build-aarch64