# CLI reference

- [Commands](#commands)
- [`novops load`](#novops-load)
- [`novops run`](#novops-run)
- [`novops completion`](#novops-completion)
- [`novops schema`](#novops-schema)
- [Built-in environment variables](#built-in-environment-variables)
- [Variables loaded by default](#variables-loaded-by-default)
- [Examples](#examples)
  - [Override default config path](#override-default-config-path)
  - [Run a sub-process](#run-a-sub-process)
  - [Specify environment without prompt](#specify-environment-without-prompt)
  - [Use built-in environment variables](#use-built-in-environment-variables)
  - [Check environment currently loaded by Novops](#check-environment-currently-loaded-by-novops)
  - [Writing .env to secure directory](#writing-env-to-secure-directory)
  - [Change working directory](#change-working-directory)
  - [Dry-run](#dry-run)

## Commands

Novops commands:

- `load` - Load a Novops environment. Output resulting environment variables to stdout or to a file is `-s` is used
- `run` - Run a command with loaded environment variables and files
- `completion` -  Output completion code for various shells
- `schema` - Output Novops confg JSON schema
- `help` - Show help and usage

All commands and flags are specified in [`main.rs`](../../src/main.rs)

## `novops load`

```
novops load [OPTIONS]
```

Load a Novops environment. Output resulting environment variables to stdout or to a file using `-s/--symlink`.

Intended usage is for redirection with `source` such as:

```sh
source <(novops load)
```

It's also possible to create a `dotenv` file in a secure directory and a symlink pointing to it with `--symlink/-s`:

```sh
novops load -s .envrc
source .envrc
```

Options:

`-c, --config <FILE>` - Configuration to use. Default: `.novops.yml`
`-e, --env <ENVNAME>` - Environment to load. Prompt if not specified.
`-s, --symlink <SYMLINK>` -  Create a symlink pointing to generated environment variable file. Implies -o 'workdir'
`-f, --format <FORMAT>` - Format for environment variables (see below)
`-w, --working-dir <DIR>` - Working directory under which files and secrets will be saved. Default to `XDG_RUNTIME_DIR` if available, or a secured temporary files otherwise. See [Security Model](./security.md) for details. 
`--dry-run` - Perform a dry-run: no external service will be called and dummy secrets are generated.

Supported environment variable formats with `-f, --format <FORMAT>`:

- `dotenv-export` output variables with `export` keywords such as
  ```
  export FOO='bar'
  ```
- `dotenv` output variables as-is such as
  ```
  FOO='bar'
  ```

## `novops run`

```sh
novops run [OPTIONS] -- <COMMAND>...
```

Run a command with loaded environment variables and files. Example: 

```
novops run -- sh
novops run -- terraform apply
```

**Always use `--` before your command to avoid `OPTIONS` being mixed-up with `COMMAND`**. For example, `novops run -e prod-rw sh -c "echo hello"` would cause Novops to interpret `-c` as `OPTIONS` rather than `COMMAND`. Note that future version of Novops may enforce `--` usage, so commands like `novops run echo foo` may not be valid anymore. 

Options:

- `-c, --config <FILE>` - Configuration to use. Default: `.novops.yml`
- `-e, --env <ENVNAME>` - Environment to load. Prompt if not specified.
- `-w, --working-dir <DIR>` - Working directory under which files and secrets will be saved. Default to `XDG_RUNTIME_DIR` if available, or a secured temporary files otherwise. See [Security Model](./security.md) for details. 
- `--dry-run` - Perform a dry-run: no external service will be called and dummy secrets are generated. `COMMAND` willl be called with dummy secrets.

## `novops completion`

```
novops completion <SHELL>
```

Output completion code for various shells. Examples: 
- bash: `source <(novops completion bash)` 
- zsh: `novops completion zsh > _novops && fpath+=($PWD) && compinit` 

Add output to your `$HOME/.<shell>rc`  file.

## `novops schema`

```
novops schema
```

Output Novops config JSON schema. 

## Built-in environment variables

CLI flags can be specified via environment variables `NOVOPS_*`:

- `NOVOPS_CONFIG` - global flag `-c, --config`
- `NOVOPS_ENVIRONMENT` - global flag `-e, --env`
- `NOVOPS_WORKDIR` - global flag `-w, --working-dir`
- `NOVOPS_DRY_RUN` - global flag `--dry-run`
- `NOVOPS_SKIP_WORKDIR_CHECK` - global flag `--skip-workdir-check`
- `NOVOPS_LOAD_SYMLINK` - load subcommand flag `-s, --symlink`
- `NOVOPS_LOAD_FORMAT` - load subcommand flag `-f, --format `
- `NOVOPS_LOAD_SKIP_TTY_CHECK` - load subcommand `--skip-tty-check`


## Variables loaded by default

Novops will load some variables by default when running `novops [load|run]`:

- `NOVOPS_ENVIRONMENT` - Name of the loaded environment

## Examples

### Override default config path

By default Novops uses `.novops.yml` to load secrets. Use `novops load -c PATH` to use another file:

```sh
novops load -c /path/to/novops/config.yml
```

### Run a sub-process

Use `novops run`

```sh
novops run -- sh
```

Use `FLAG -- COMMAND...` to provide flags:

```sh
novops run -e dev -c /tmp/novops.yml -- run terraform apply
```

### Specify environment without prompt

Use `novops load -e ENV` to load environment without prompting

```sh
source <(novops load -e dev)
```

### Use built-in environment variables

Sometime you want to change behavior according to environment variables, such as running Novops on CI.

Use built-in environment variables:

```sh
# Set environment variable
# Typically done via CI config or similar
# Using export for example
export NOVOPS_ENVIRONMENT=dev
export NOVOPS_LOAD_SYMLINK=/tmp/.env

# Novops will load dev environment and create /tmp/.env symlink
novops load 
```

Equivalent to

```sh
novops load -e dev -s /tmp/.env
```

### Check environment currently loaded by Novops

Novops exposes some variables by default (eg. `NOVOPS_ENVIRONMENT`). You can use them to perform some specific actions.

Simple example: show loaded environment

```sh
novops run -e dev -- sh -c 'echo "Current Novops environment: $NOVOPS_ENVIRONMENT"'
```

You can leverage `NOVOPS_ENVIRONMENT` to change behavior on certain environments, such as avoiding destructive action in Prod:

```sh
# Failsafe: if current environment is prod or contains 'prod', exit with error
if [[ $NOVOPS_ENVIRONMENT == *"prod"* ]]; then
  echo "You can't run this script in production or prod-like environments!"
  exit 1
fi

# ... some destructive actions
make destroy-all
```

`NOVOPS_ENVIRONMENT` is automayically loaded:

```sh
novops run -e prod -- ./destroy-all.sh  # Won't work
novops run -e dev -- ./destroy-all.sh   # OK
```

You may instead add a custom `MY_APP_ENVIRONMENT` on each environment but it's less convenient. 

### Writing .env to secure directory

Without Novops, you'd write some `.env` variable file directly to disk and source it. But writing data directly to disk may represent a risk. 

`novops load -s SYMLINK` creates a symlink pointing to secret file stored securely in a `tmpfs` directory.

```sh
# Creates symlink .envrc -> /run/user/1000/novops/myapp/dev/vars
novops load -s .envrc

source .envrc # source it !

cat .envrc
# export HELLO_WORLD='Hello World!'
# export HELLO_FILE='/run/user/1000/novops/myapp/dev/file_...'
```

### Change working directory

Novops uses [`XDG_RUNTIME_DIR` by default](../advanced/security.md) as secure working directory for writing files. You can change working directory with `novops load -w`. **No check is performed on written-to directory. Make sure not to expose secrets this way.**

```sh
novops load -w "$HOME/another/secure/directory"
```

### Dry-run

Mostly used for testing, dry-run will only parse config and generate dummy secrets without reading or writing any actual secret value. 

```sh
novops load --dry-run
```