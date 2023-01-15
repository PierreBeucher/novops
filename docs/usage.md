# Novops usage

- [Novops usage](#novops-usage)
  - [Shell](#shell)
  - [Docker](#docker)
  - [GitLab CI](#gitlab-ci)
- [Where are secret files stored?](#where-are-secret-files-stored)
  
## Shell

For local usage, [`direnv`](https://direnv.net/) usage is [recommended for security reasons](./novops-direnv.md). 

```sh
# Generate env file at .envrc
# direnv will automatically load it
novops -e dev -s .envrc
```

It's also possible to source manually when using `direnv` is not practical or necessary (such as in a Docker container or CI job):

```sh
# Source manually
# But variables won't be unset unless done manually
novops -e dev -s .env && source .env
```

## Docker

_NOTE: Novops Docker image is not yet published but [Dockerfile is available at root](../Dockerfile)_

Include in your Dockerfile with:

```Dockerfile
FROM novops

FROM alpine
COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops
```

Then run a container with your `.novops.yml` such as:

```sh
# Mount novops config in container and run novops
docker run -it -v $PWD/.novops.yml:/novops-config.yml IMAGE
$ novops -c /novops-config.yml && source /tmp/.novops/vars

# Alternatively mount entire directory
docker run -it -v $PWD:/myapp -w /myapp IMAGE
$ novops && source /tmp/.novops/vars
```

## GitLab CI

Load Novops at beginning of job:

```yaml
job-using-novops:
  # ...
  script:
    - novops load -e dev -s .env && source .env
    # Variables will be available for subsequent commands
    - make test 
```

# Where are secret files stored?

Secret files will be stored under [`XDG_RUNTIME_DIR`](https://askubuntu.com/questions/872792/what-is-xdg-runtime-dir), a user-specific folder in which to store small temporary files available on most Linux distributions. It provides a basic layer of security as only the current Linux user can access it and files are usually stored in-memory. 

If `XDG_RUNTIME_DIR` is not available, a user-specific folder in `/tmp` will be created and used instead (and a warning issued). This alternative is less secure but still better than a word-readable file. 

Alternatively, you can specify `-w PATH` flag to use a custom directory. The directory must exists, and it's recommended to ensure user-only permissions on it (i.e. `0600`/`-rw------`)