default: target/release/tako

target/release/tako: Cargo.toml Cargo.lock src/*.rs
	cargo build --release
	strip target/release/tako

site/index.html: docs/*.md mkdocs.yml
	mkdocs build

docs: site/index.html

# Don't do a Cargo clean, rebuilding Rust deps is expensive.
# Run "cargo clean" when you want that.
clean:
	rm target/release/tako
	rm -fr site
