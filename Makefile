docker:
	docker buildx build . -t novops:local --load 

build:
	cargo build --release --target x86_64-unknown-linux-musl

test:
	docker-compose -f tests/docker-compose.yml up -d
	cargo test