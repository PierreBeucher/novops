# Static build using musl
# clux/muslrust provides static build of popular libraries
# allowing a fully static binary as output
FROM docker.io/clux/muslrust:1.86.0-stable as builder

WORKDIR /build

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src src/

RUN cargo build --release --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/novops /novops

FROM scratch

COPY --from=builder /novops /novops