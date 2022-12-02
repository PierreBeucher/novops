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

Integration tests are run using cargo and external dependencies as Docker containers (Hashicorp Vault, etc.)


```sh
# Run Docker Compose stack and run tests
make test
```

Tests are run on CI for any non-`master` branch using the same procedure. 

## Updating dependencies

This command regenerates the Cargo.nix and as such should be run everytime Cargo.lock is changed. It will fail on CI if not done. 

```sh
nix run github:cargo2nix/cargo2nix/release-0.11.0
```