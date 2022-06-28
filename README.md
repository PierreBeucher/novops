# Novops

A platform agnostic secret aggregator for CI and development environments.

## Usage

Novops will load configs and secrets defined in config (`.novops.yml` by default) as environment variable and files.

## Simple example

Consider example `.novops.yml`

```yaml
name: myapp

environments:
  dev:
    variables:
      APP_HOST: "127.0.0.1:8080"

    files: 
      npm_token:
        dest: ".token"
        content: 
          bitwarden:
            entry: "Dev Secret Token"
            field: notes
      
      foo:
        content: "bar"
```

Run command

```sh
novops -e dev -w ".novops"
source ".novops/vars"
```

Will result in:

- File `.token` created with content from BitWarden entry `Staging Secret Token`
- File `/run/user/1000/novops/myapp/dev/files/foo` created with content `bar`
  - Without `dest` is specified, file is created in XDG Runtime Directory (an environment variable is set pointing to it, see below)
- Variables exported:
  ```sh
  # as stated in config
  APP_HOST="127.0.0.1:8080"

  # Every file in config comes with a variable pointing to its path
  NOVOPS_FILE_DEV_NPM_TOKEN=/dir/running/novops/.token
  NOVOPS_FILE_DEV_FOO=XDG_RUNTIME_DIR=/run/user/1000/novops/myapp/dev/files/foo
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

## AVailable secret providers

- Bitwarden
- _Soon: Hashicorp Vault_

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

TODO
