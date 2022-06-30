docker:
	docker buildx build . 

build:
	cargo build --release --target x86_64-unknown-linux-musl