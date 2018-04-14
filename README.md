# Tako

Tako: take container image.

## Building

    rustup target add x86_64-unknown-linux-musl
    cargo build --release --target=x86_64-unknown-linux-musl
    target/x86_64-unknown-linux-musl/release/tako
