# syntax=docker/dockerfile:experimental
FROM rust:1.61.0-alpine3.16 as builder

RUN apk update && apk add --no-cache musl-dev

WORKDIR /novops

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/novops/target \
    cargo build --release --target x86_64-unknown-linux-musl

# FROM alpine:3.16

# COPY --from=builder /novops/target/release/novops /usr/local/bin/novops

# CMD ["novops"]
