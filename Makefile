.PHONY: dockers
docker:
	docker buildx build . -t novops:local --load 

.PHONY: build
build:
	docker buildx build . -o type=local,dest=build

.PHONY: test-docker
test-docker:
	docker-compose -f tests/docker-compose.yml up -d

.PHONY: test
test: test-docker
	cargo test