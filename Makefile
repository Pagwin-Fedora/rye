VER=src
debug:
	cargo build
release:
	cargo build --release
container:
	-rm rye; 
	cargo build --release --target=x86_64-unknown-linux-musl
	mv target/x86_64-unknown-linux-musl/release/rye .
	docker build -t rye:$(VER) .
