# Build container image and binary via Podman
# Keep cache under build/cache via bind mounts 
# (instead of BuildKit cache mount which is harder to cache)
.PHONY: build-cache-dir
build-cache-dir:
	mkdir -p build build/cache build/cache/cargo-target build/cache/cargo-reg

.PHONY: build-image
build-image: build-cache-dir
	podman build \
		-v ${PWD}/build/cache/cargo-target:/build/target \
		-v ${PWD}/build/cache/cargo-reg:/root/.cargo/registry \
		-t novops:local \
		.
	podman save novops:local -o build/image.tar --format=oci-archive

.PHONY: build-binary
build-binary: build-cache-dir
	podman build \
		-v ${PWD}/build/cache/cargo-target:/build/target \
		-v ${PWD}/build/cache/cargo-reg:/root/.cargo/registry \
		-o type=local,dest=build \
		.
	
	zip -j build/novops.zip build/novops
	sha256sum build/novops.zip > build/novops.zip.sha256sum

.PHONY: build-nix
build-nix:
	nix build -o build/nix

.PHONY: test
test: test-docker test-doc test-cargo

.PHONY: test-docker
test-docker:
	podman-compose -f tests/docker-compose.yml up -d

.PHONY: test-cargo
test-cargo:
	cargo test

# Fails if doc is not up to date with current code
.PHONY: test-doc
test-doc: doc
	git diff --exit-code docs/schema/config-schema.json

# Build doc with mdBook and json-schema-for-humans
# See:
# - https://github.com/actions/starter-workflows/blob/main/pages/mdbook.yml
# - https://coveooss.github.io/json-schema-for-humans/#/
.PHONY: doc
doc:
	mdbook build ./docs/
	cargo run -- schema > docs/schema/config-schema.json
	generate-schema-doc --config footer_show_time=false --config link_to_reused_ref=false --config expand_buttons=true docs/schema/config-schema.json  docs/book/config/schema.html

doc-serve:
	(cd docs/ && mdbook serve -o)

# Clean caches and temporary directories
clean:
	echo "todo"


# Publish image to Docker Hub for release or locally for testing
DOCKER_REPOSITORY ?= oci-archive:/tmp/novops-image-archive
GITHUB_REF_NAME ?= local
.PHONY: docker-publish
docker-publish:
	podman load -i build/image.tar
	podman push novops:local ${DOCKER_REPOSITORY}:${GITHUB_REF_NAME}
	podman push novops:local ${DOCKER_REPOSITORY}:latest

.PHONY: release-tag
release-tag:
	npx release-please github-release --repo-url https://github.com/PierreBeucher/novops --token=${GITHUB_TOKEN}

.PHONY: release-pr
release-pr:
	npx release-please release-pr --repo-url https://github.com/PierreBeucher/novops --token=${GITHUB_TOKEN}


RUNNER_ARCH ?= X64
RUNNER_OS ?= Linux
GITHUB_REF_NAME ?= vx.x.x
.PHONY: release-artifacts
release-artifacts:
	cp build/novops.zip build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip
	cp build/novops.zip.sha256sum build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip.sha256sum
	gh release upload ${GITHUB_REF_NAME} \
          build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip \
          build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip.sha256sum