# Development guide

## Build

Novops is built with Docker BuildKit. Built binary is fully static. See [Dockerfile](../Dockerfile).

```sh
# Result in ./build/novops
make build
```

## Run test

Integration tests are run when possible real service, falling back to emulator or dry-run when not practical:
- AWS: [LocalStack](https://localstack.cloud) server
- Hashivault: [Vault Docker image](https://hub.docker.com/_/vault)
- Google Cloud: `--dry-run` mode 
- Azure: `--dry-run` mode 

```sh
# Run Docker Compose stack and run tests
make test

# Alternatively, run Docker stack and specific tests
make test-docker
RUST_LOG=novops=debug cargo test --test test_aws -- --nocapture
```

Tests are run on CI for any non-`master` branch using the same procedure. 

## Updating dependencies

This command regenerates the Cargo.nix and as such should be run everytime Cargo.lock is changed. It will fail on CI if not done. 

```sh
nix run github:cargo2nix/cargo2nix/unstable
```

## Releasing

`release-please` should create/update Release PRs automatically on `main` changes. Merging automatically creates release and related artifacts. 