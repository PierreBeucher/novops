# Build all cross targets
# Use different target dir to avoid glibc version error
# See https://github.com/cross-rs/cross/issues/724
.PHONY: build-cross
build-cross:
	# Can't include darwin targets as not possible to use it on CI directly
	# cross build --target x86_64-apple-darwin --target-dir target/cross/x86_64-apple-darwin
	# cross build --target aarch64-apple-darwin --target-dir target/cross/aarch64-apple-darwin
	cross build --target x86_64-unknown-linux-musl --target-dir target/cross/x86_64-unknown-linux-musl
	cross build --target aarch64-unknown-linux-musl --target-dir target/cross/aarch64-unknown-linux-musl

.PHONY: build-nix
build-nix:
	nix build -o build/nix

.PHONY: test
test: test-prepare test-doc test-clippy test-cargo test-cli test-install test-teardown

.PHONY: test-prepare
test-prepare:
	tests/scripts/test-docker-prepare.sh

.PHONY: test-teardown
test-teardown:
	tests/scripts/test-docker-teardown.sh

.PHONY: test-cargo
test-cargo:
	cargo test

test-cli:
	tests/cli/test-usage.sh

.PHONY: test-clippy
test-clippy:
	cargo clippy -- -D warnings

# Fails if doc is not up to date with current code
.PHONY: test-doc
test-doc: doc
	git diff --exit-code docs/schema/config-schema.json

.PHONY: test-install
test-install:
	tests/install/test-install.sh

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


#
# Relase 
#

# Publish image to Docker Hub for release or locally for testing
DOCKER_REPOSITORY ?= docker://docker.io/crafteo/novops
GITHUB_REF_NAME ?= local # Set by GitHub Actions
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

# Publish artifact on GitHub release
RUNNER_ARCH ?= X64
RUNNER_OS ?= Linux
GITHUB_REF_NAME ?= local # Set by GitHub Actions
.PHONY: release-artifacts
release-artifacts:
	cp build/novops.zip build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip
	cp build/novops.zip.sha256sum build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip.sha256sum
	gh release upload ${GITHUB_REF_NAME} \
          build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip \
          build/novops-${RUNNER_ARCH}-${RUNNER_OS}.zip.sha256sum
