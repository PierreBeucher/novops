# Development

## Build

Plain binary:

```sh
carbo build 
```

Docker image (using BuildKit):

```sh
docker buildx build .
```

## Run test

Use cargo and external dependencies as Docker containers: Hashicorp Vault, [LocalStack](https://localstack.cloud), ...


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
nix run github:cargo2nix/cargo2nix/release-0.11.0
```