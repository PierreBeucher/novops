.PHONY: build
build:
	docker buildx build . -o type=local,dest=build

.PHONY: test-docker
test-docker:
	docker-compose -f tests/docker-compose.yml up -d --wait

.PHONY: test
test: test-docker
	cargo test

doc:
	mdbook build ./docs/
	cargo run -- schema > docs/schema/config-schema.json 
	generate-schema-doc docs/schema/config-schema.json  docs/schema/index.html

doc-serve:
	(cd docs/ && mdbook serve -o)
