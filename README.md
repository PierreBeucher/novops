# Novops

A platform agnostic secret aggregator for CI and development environments.

## Features

- Load secrets and config as file or environment variables in most shells
- Integrate with various secret providers: Hashicorp Vault, AWS, BitWarden...
- Manage multiple environments
- Integrate with CI to help reduce drift between CI and local environment

## Usage

Novops load configs and secrets defined in `.novops.yml` to use them as file and/or environment variables. 

### Simple example

Consider example `.novops.yml`

```yaml
name: myapp

environments:
  dev:
    variables:
      # Plain string
      - name: APP_URL
        value: "http://127.0.0.1:8080"

      # Retrieve secret from Hashicorp Vault using KV v2 Secret Engine
      - name: APP_PASSWORD
        value:
          hvault_kv2:
            mount: "secret"
            path: "myapp/dev/creds"
            entry: "password"

    files: 
      # Retrieve secret from BitWarden and save it to file
      # File path will be exposed via env var APP_TOKEN
      - name: APP_TOKEN
        content: 
          bitwarden:
            entry: "Dev Secret Token"
            field: notes
```

Run Novops with

```sh
# Load dev config and source env variables in current shell
# Creates a symlink .env -> $XDG_RUNTIME_DIR/.../vars to keep secrets safe and allow easy sourcing
novops -e dev -s .env && source .env

echo $APP_URL 
# 127.0.0.1:8080

echo $APP_PASSWORD
# s3cret

cat $APP_TOKEN
# SomeTokenValue

# Files are created securely under XDG Runtime Dir by default
echo $APP_TOKEN
# /run/user/1000/novops/myapp/dev/file_APP_TOKEN

```

### Bash / Shell

```
novops -e dev -w ".novops"; source ".novops/vars"
```

### Docker

Include in your Dockerfile with:

```Dockerfile
FROM novops

FROM alpine
COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops
```

Then use with bash/shell in container:

```
docker run -it -v $PWD/.novops.yml:/novops-config.yml
$ novops -c /novops-config.yml -w /tmp/.novops; source /tmp/.novops/vars
```

### Nix

TODO

## Modules usage

Quick reference and example of available Modules. A Module allows you to load files and environment variables from a secret provider (or any external source).

### Hashicorp Vault

[Key Value Version 2](https://www.vaultproject.io/docs/secrets/kv/kv-v2) with variables and files:

```yaml
environment:
  dev:
    variables:
    - name: APP_PASSWORD
      value:
        hvault_kv2:
          mount: "secret"
          path: "myapp/dev/creds"
          entry: "password"

    files:
    - name: SECRET_TOKEN
      dest: .token
      content:
        hvault_kv2:
          path: "myapp/dev/creds"
          entry: "token"
```

### AWS

Generate temporary [IAM Role credentials with AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html):

```yaml
environments:
  dev:
    # Output variables to assume IAM Role:
    # AWS_ACCESS_KEY_ID
    # AWS_SECRET_ACCESS_KEY
    # AWS_SESSION_TOKEN
    aws:
      assume_role:
        role_arn: arn:aws:iam::12345678910:role/my_dev_role
        source_profile: novops
```

### BitWarden

_Experimental module, requires BitWarden CLI installed locally_

```yaml
environments:
  dev:
    files: 
    - name: ssh-key
      content:
        bitwarden:
          entry: Some SSH Key entry
          field: notes
```

## Development

### Build

Plain binary:

```sh
carbo build 
```

Docker image (using BuildKit):

```sh
docker buildx build .
```

### Run test

Integ tests are run within Docker to have a similar environment locally and on CI. 

Run tests locally (via `docker exec` within a Rust container):

```
make test-docker
```

Tests are run on CI for any non-`master` branch. 

### Updating dependencies

We use cargo2nix that can build dependencies separately (it is more granular than nixpkgs' solution) with the inconvenient that now one needs

```sh
nix run github:cargo2nix/cargo2nix
```

### Advanced concepts

- [Internal code architecture](docs/internals.md)

## Contributing

Feel free to open a Pull Request to contribute code, tests, doc...