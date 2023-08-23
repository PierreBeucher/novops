# Docker & Podman

- [Run containers](#run-containers)
- [Compose](#compose)
- [Build images](#build-images)

## Run containers 

Load environment variables directly into containers:

```sh
docker run -it --env-file <(novops load -f dotenv -e dev) alpine sh
podman run -it --env-file <(novops load -f dotenv -e dev) alpine sh
```

`novops load -f dotenv` generates an env file output compatible with Docker and Podman.

## Compose 

Use [Docker Compose](https://docs.docker.com/compose/), [podman-compose](https://github.com/containers/podman-compose) or another tool compatible with [Compose Spec](https://github.com/compose-spec/compose-spec)


Generate a `.env` file

```sh
novops load -s .env
```

And use it on Compose file

```yaml
services:
  web:
    image: 'webapp:v1.5'
    env_file: .env
```

[See Compose Spec for details](https://github.com/compose-spec/compose-spec/blob/master/05-services.md)

## Build images

Include `novops` in your Dockerfile such a:

```Dockerfile
# Multi-stage build to copy novops binary from existing image
FROM crafteo/novops:0.7.0 AS novops

# Final image where novops is copied
FROM alpine AS app

COPY --from=novops /novops /usr/local/bin/novops
```