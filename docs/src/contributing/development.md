# Development guide

Every command below must be run under [Nix Flake development shell](https://nixos.wiki/wiki/Flakes):

```sh
nix develop
```

All commands are CI-agnostic: they work the same locally and on CI by leveraging Nix and cache-reuse. If it works locally, it will work on CI.

- [Build](#build)
- [Test](#test)
  - [Running non-integration tests](#running-non-integration-tests)
  - [Runnning integration tests](#runnning-integration-tests)
- [Doc](#doc)
- [Release](#release)

## Build

For quick feedback, just run 

```
cargo build
cargo build -j 6
```

Novops is built for multiple platforms using `cross`:

```sh
task build-cross
```

For Darwin (macOS), you must build Darwin Cross image yourself (Apple does not allow distribution of macOS SDK required for cross-compilation, but you can download it yourself and package Cross image):

- [Download XCode](https://developer.apple.com/xcode/resources/) (see also [here](https://xcodereleases.com/))
- Follow [osxcross](https://github.com/tpoechtrager/osxcross) instructions to package macOS SDK 
  - At time of writing this doc, latest supported version of XCode with osxcross was 14.1 (SDK 13.0)
- Use [https://github.com/cross-rs/cross] and [cross-toolchains](https://github.com/cross-rs/cross-toolchains) to build your image from Darwin Dockerfile
  - For example:
    ```sh
    # Clone repo and submodules
    git clone https://github.com/cross-rs/cross
    cd cross
    git submodule update --init --remote

    # Copy SDK to have it available in build context
    cd docker
    mkdir ./macos-sdk
    cp path/to/sdk/MacOSX13.0.sdk.tar.xz ./macos-sdk/MacOSX13.0.sdk.tar.xz

    # Build images
    docker build -f ./cross-toolchains/docker/Dockerfile.x86_64-apple-darwin-cross \
      --build-arg MACOS_SDK_DIR=./macos-sdk \
      --build-arg MACOS_SDK_FILE="MacOSX13.0.sdk.tar.xz" \
      -t x86_64-apple-darwin-cross:local .

    docker build -f ./cross-toolchains/docker/Dockerfile.aarch64-apple-darwin-cross \
      --build-arg MACOS_SDK_DIR=./macos-sdk \
      --build-arg MACOS_SDK_FILE="MacOSX13.0.sdk.tar.xz" \
      -t aarch64-apple-darwin-cross:local \
      .
    ```

## Test

Tests are run on CI using procedure described below. It's possible to run them locally as well under a `nix develop` shell.

### Running non-integration tests


These tests dot not require anything special and can be run as-is:

```sh
task test-doc
task test-clippy
task test-cli
task test-install
```

### Runnning integration tests

Requirements:
- Running a `nix develop` shell
- Azure account
- GCP account

Integration tests run with real services, preferrably in containers or using dedicated Cloud account:
- AWS: [LocalStack](https://localstack.cloud) container (AWS emulated in a container)
- Hashicorp Vault: [Vault container](https://hub.docker.com/_/vault)
- Google Cloud: GCP account
- Azure: Azure account

Integration test setup is fully automated but **may create real Cloud resources**. Run:

```sh
task test-setup
```

See `tests/setup/pulumi`. 

**Remember to `task teardown` after running integration tests.** Cost should be negligible if you teardown infrastructure right after running tests. Cost should still be negligible even if you forget to teardown as only free or cheap resources are deployed, but better to do it anyway. 

```sh
# Setup containers and infrastructure and run all tests
# Only needed once to setup infra
# See Taskfile.yml for details and fine-grained tasks
task test-setup

# Run tests
task test-integ

# Cleanup resources to avoid unnecessary cost
task test-teardown
```

## Doc

Doc is built with [`mdBook`](https://github.com/rust-lang/mdBook) and JSON Schema generated from [schemars](https://docs.rs/schemars/latest/schemars/).

Doc is published from `main` branch by CI

```sh
# Build doc
task doc

# Serve at locahost:3000
tasl doc-serve
```

## Release

`release-please` should create/update Release PRs automatically on `main` changes. After merge, release tag and artifacts must be created locally:

Run `cross` Nix shell

```sh
nix develop .#cross
```

Create release

```sh
# GITHUB_TOKEN must be set with read/write permissions 
# on Contents and Pull requests
export GITHUB_TOKEN=xxx 

git checkout <release_commit_sha>

# git checkout main && git pull right after merge should be OK

hack/create-release.sh
```

Notes: 
- Release may take some time as it will cross-build all Novops binaries before running [`release-please`](https://github.com/googleapis/release-please)
- MacOS build image must be available locally (see Build above)