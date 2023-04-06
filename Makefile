VER=src
debug:
	cargo build
release:
	cargo build --release
container:
	-rm rye
	cargo build --release --target=x86_64-unknown-linux-musl
	mv target/x86_64-unknown-linux-musl/release/rye .
	docker buildx build -t rye:$(VER) .

rye_$(VER)_cont.tar: container
	docker image save -o rye_$(VER)_cont.tar
