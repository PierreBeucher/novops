# Development guide

Every command below must be run under [Nix Flake development shell](https://nixos.wiki/wiki/Flakes):

```
nix develop
```

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

## Update JSON Schema

JSON Schema is generated from [schemars](https://docs.rs/schemars/latest/schemars/)

```
make doc
```

## Serve documentation locally

Doc is built with [`mdBook`](https://github.com/rust-lang/mdBook)

```
make doc-serve
```

## Releasing

`release-please` should create/update Release PRs automatically on `main` changes. Merging automatically creates release and related artifacts. 