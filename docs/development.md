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

- Run Github Action workflow `Tag release` to push new tag with changelogs
- Manually create a release from Git tag. Workflow `Publish release` will automatically add `novops` assets.