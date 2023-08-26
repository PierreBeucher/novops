# Development guide

Every command below must be run under [Nix Flake development shell](https://nixos.wiki/wiki/Flakes):

```
nix develop
```

All commands are CI-agnostic: they work the same locally and on CI by leveraging Nix and cache-reuse. If it works locally, it will work on CI.

- [Build](#build)
- [Test](#test)
- [Doc](#doc)
- [Release](#release)

## Build

Novops is built in multiple flavors:

- Static binary (Podman local output)
  ```sh
  # Result in ./build/novops
  make build-image
  ```
- Docker image (Podman)
  ```
  make build-image
  ``` 
- Nix
  ```
  make build-nix
  ``` 

## Test

Integration tests are run when possible with real services, falling back to emulator or dry-run when not practical:
- AWS: [LocalStack](https://localstack.cloud) server
- Hashivault: [Vault Docker image](https://hub.docker.com/_/vault)
- Google Cloud: `--dry-run` mode 
- Azure: `--dry-run` mode 

```sh
# Run Compose stack and run tests
make test

# Alternatively, run Docker stack and specific tests
make test-docker
RUST_LOG=novops=debug cargo test --test test_aws -- --nocapture
```

Tests are run on CI for any non-`master` branch using the same procedure.

## Doc

Doc is built with [`mdBook`](https://github.com/rust-lang/mdBook) and JSON Schema generated from [schemars](https://docs.rs/schemars/latest/schemars/).

Doc is published from `main` branch by CI

```sh
# Build doc
make doc

# Serve at locahost:3000
make doc-serve
```

## Release

`release-please` should create/update Release PRs automatically on `main` changes. Merging automatically creates release and related artifacts. 