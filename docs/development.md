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

Integ tests are run within Docker to have a similar environment locally and on CI. 

Run tests locally (via `docker exec` within a Rust container):

```
make test-docker
```

Tests are run on CI for any non-`master` branch. 

## Updating dependencies

This command regenerates the Cargo.nix and as such should be run everytime Cargo.lock is changed. It will fail on CI if not done. 

```sh
nix run github:cargo2nix/cargo2nix/release-0.11.0
```