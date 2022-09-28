# Usage

## Possible usages and examples

### Bash / Shell

With [`direnv`](https://direnv.net/) 

```sh
# Generate env file at .envrc
# direnv will automatically load it
novops -e dev -s .envrc
```

It's also possible to source manually:

```sh
# Source manually
# But variables won't be unset unless done manually
novops -e dev -s .myenvfile && source .myenvfile
```

### Docker

Include in your Dockerfile with:

```Dockerfile
FROM novops

FROM alpine
COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops
```

Then use with bash/shell in container:

```sh
docker run -it -v $PWD/.novops.yml:/novops-config.yml
$ novops -c /novops-config.yml -w /tmp/.novops; source /tmp/.novops/vars
```

### Nix

TODO

## Where are secret files stored?

Secret files will be stored under [`XDG_RUNTIME_DIR`](https://askubuntu.com/questions/872792/what-is-xdg-runtime-dir) a secured folder available on most Linux distributions. 

If `XDG_RUNTIME_DIR` is not available, a secured temporary folder will be created for current user (and a warning issued). This alternative is less secure but still better than a word-readable file. 