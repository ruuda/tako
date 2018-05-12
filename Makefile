default: target/release/tako

target/release/tako: Cargo.toml Cargo.lock src/*.rs
	cargo build --release
	strip target/release/tako

# Don't do a Cargo clean, rebuilding Rust deps is expensive.
# Run "cargo clean" when you want that.
clean:
	rm target/release/tako
