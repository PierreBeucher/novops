# syntax=docker/dockerfile:experimental
FROM rust:1.61.0-alpine3.16 as builder

WORKDIR /novops
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/novops/app/target \
    cargo build --release

FROM alpine:3.16

COPY --from=builder /novops/target/release/novops /usr/local/bin/novops

CMD ["novops"]
