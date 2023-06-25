# Static build using musl
# clux/muslrust provides static build of popular libraries
# allowing a fully static binary as output
FROM clux/muslrust:1.69.0-stable as builder

WORKDIR /build

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src src/

RUN --mount=type=cache,target=/usr/local/cargo/registry,id=novops-reg-static \
    --mount=type=cache,target=/build/target,id=novops-build-static \
    cargo build --release --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/novops /novops

FROM scratch

COPY --from=builder /novops /novops