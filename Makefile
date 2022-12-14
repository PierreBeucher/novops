.PHONY: dockers
docker:
	docker buildx build . -t novops:local --load 

.PHONY: build
build:
	cargo build --release --target x86_64-unknown-linux-musl

.PHONY: test-docker
test-docker:
	docker-compose -f tests/docker-compose.yml up -d

.PHONY: test
test: test-docker
	cargo test