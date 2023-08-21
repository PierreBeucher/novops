# Advanced CLI usage

- [Advanced CLI usage](#advanced-cli-usage)
  - [Override default config path](#override-default-config-path)
  - [Run a sub-process](#run-a-sub-process)
  - [Specify environment without prompt](#specify-environment-without-prompt)
  - [Writing .env to secure directory](#writing-env-to-secure-directory)
  - [Change working directory](#change-working-directory)
  - [Dry-run](#dry-run)

## Override default config path

By default Novops uses `.novops.yml` to load secrets. Use `novops load -c PATH` to use another file:

```sh
novops load -c /path/to/novops/config.yml
```

## Run a sub-process

Use `novops run`

```sh
novops run sh
```

Use `FLAG -- COMMAND...` to provide flags:

```sh
novops run -e dev -c /tmp/novops.yml -- run terraform apply
```

## Specify environment without prompt

Use `novops load -e ENV` to load environment without prompting

```sh
source <(novops load -e dev)
```

## Writing .env to secure directory

You can write `.env` variable file to to disk in a secure directory and source it later. **This usage is not recommended** as writing data to disk may represent a risk. 

`novops load -s SYMLINK` creates a symlink pointing to secret file, easing usage without compromising security:

```sh
# Creates symlink .envrc -> /run/user/1000/novops/myapp/dev/vars
novops load -s .envrc

source .envrc # source it !

cat .envrc
# export HELLO_WORLD='Hello World!'
# export HELLO_FILE='/run/user/1000/novops/myapp/dev/file_...'
```

## Change working directory

Novops uses [`XDG_RUNTIME_DIR` by default](../advanced/security.md) as secure working directory for writing files. You can change working directory with `novops load -w`. **No check is performed on written-to directory. Make sure not to expose secrets this way.**

```sh
novops load -w "$HOME/another/secure/directory"
```

## Dry-run

Mostly used for testing, dry-run will only parse config and generate dummy secrets without reading or writing any actual secret value. 

```sh
novops load --dry-run
```