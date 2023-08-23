# CLI reference

- [Commands](#commands)
- [`novops load`](#novops-load)
- [`novops run`](#novops-run)
- [`novops completion`](#novops-completion)
- [`novops schema`](#novops-schema)
- [Examples](#examples)
  - [Override default config path](#override-default-config-path)
  - [Run a sub-process](#run-a-sub-process)
  - [Specify environment without prompt](#specify-environment-without-prompt)
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
novops run [OPTIONS] <COMMAND>...
```

Run a command with loaded environment variables and files. Example: 

```
novops run sh
novops run -- terraform apply
```

`-c, --config <FILE>` - Configuration to use. Default: `.novops.yml`
`-e, --env <ENVNAME>` - Environment to load. Prompt if not specified.
`-w, --working-dir <DIR>` - Working directory under which files and secrets will be saved. Default to `XDG_RUNTIME_DIR` if available, or a secured temporary files otherwise. See [Security Model](./security.md) for details. 
`--dry-run` - Perform a dry-run: no external service will be called and dummy secrets are generated. `COMMAND` willl be called with dummy secrets.

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

## Examples

### Override default config path

By default Novops uses `.novops.yml` to load secrets. Use `novops load -c PATH` to use another file:

```sh
novops load -c /path/to/novops/config.yml
```

### Run a sub-process

Use `novops run`

```sh
novops run sh
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

### Writing .env to secure directory

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