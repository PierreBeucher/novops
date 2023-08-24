# Shell usage examples (sh, bash, zsh...)

- [Source into current shell](#source-into-current-shell)
- [Run sub-process](#run-sub-process)
- [Create `dotenv` file in secure directory with symlink](#create-dotenv-file-in-secure-directory-with-symlink)

## Source into current shell

Source directly into your shell

```sh
source <(novops load)
```

You can also create an alias such as

```sh
alias nload="source <(novops load)"
```

Run `unload` to unload to unset variables sourced by `novops load`

```
unload
```

`unload` is a function declared when using `source <(novops load)`. It `unset` itself and all variables loaded by Novops. 

## Run sub-process

Run a sub-process or command loaded with environment variables:

```sh
novops run -- terraform apply
```

This will ensure secrets are only exists in memory for as long as command run.

## Create `dotenv` file in secure directory with symlink

_Note: this method is not recommended as it will write secrets to disks as `dotenv` file._ If possible, prefer using one of the method above. If you need to pass a file to a process such as Docker, use syntax `COMMAND --env-file <(novops load -f dotenv -e dev) ARG` to pass environment as file to process without writing to disk.

Load secrets and create a `.env -> /run/user/1000/novops/.../vars` symlink pointing to dotenv file sourceable into your environment. 

```sh
novops load -s .env

# .env is a symlink
# There's no risk commiting to Git
# Source it !
source .env
```