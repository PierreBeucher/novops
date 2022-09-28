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

We use cargo2nix that can build dependencies separately (it is more granular than nixpkgs' solution) with the inconvenient that now one needs

```sh
nix run github:cargo2nix/cargo2nix
```