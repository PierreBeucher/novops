# Shell usage examples (sh, bash, zsh...)

- [Source into current shell](#source-into-current-shell)
- [Run sub-process](#run-sub-process)
- [Create `dotenv` file in protected directory with symlink](#create-dotenv-file-in-protected-directory-with-symlink)

## Source into current shell

Source into your shell

```sh
# bash
source <(novops load)

# zsh / ksh 
source =(novops load)

# dash
novops load -s .envrc
. ./.envrc

# fish
source (novops load | psub)
```

You can also create an alias such as

```sh
alias nload="source <(novops load)"
```

## Run sub-process

Run a sub-process or command loaded with environment variables:

```sh
# Run terraform apply
novops run -- terraform apply

# Run a sub-shell
novops run -- sh
```

This will ensure secrets are only exists in memory for as long as command run.

## Create `dotenv` file in protected directory with symlink

Load secrets and create a `.env -> /run/user/1000/novops/.../vars` symlink pointing to dotenv file sourceable into your environment. 

```sh
novops load -s .envrc

# .env is a symlink
# There's no risk commiting to Git
# Source it !
source .env
```