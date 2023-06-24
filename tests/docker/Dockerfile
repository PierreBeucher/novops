# Intermediate image to download novops in
FROM alpine AS novops

# See https://github.com/novadiscovery/novops/releases for latest version
ARG NOVOPS_VERSION=0.6.0

RUN apk add curl unzip

RUN curl -L "https://github.com/novadiscovery/novops/releases/download/v${NOVOPS_VERSION}/novops-X64-Linux.zip" -o novops.zip && \
    unzip novops.zip && \
    mv novops /usr/local/bin/novops

# Final image where novops is added
FROM alpine AS app

# ...

COPY --from=novops /usr/local/bin/novops /usr/local/bin/novops

# ... 