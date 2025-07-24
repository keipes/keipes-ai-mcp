FROM --platform=$BUILDPLATFORM rust:latest AS builder

ARG TARGETOS
ARG TARGETARCH
ARG BUILDPLATFORM
ARG TARGETPLATFORM

RUN echo "Building for $TARGETOS/$TARGETARCH on $BUILDPLATFORM/$TARGETPLATFORM"

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# RUN cargo build --release

# RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/* && rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/* && rustup target add aarch64-unknown-linux-gnu; \
        fi

# cross-compile if possible
RUN if [ "$TARGETARCH" = "arm64" ]; then \
        export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc && \
        export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc && \
        RUST_TARGET=aarch64-unknown-linux-gnu; \
    elif [ "$TARGETARCH" = "amd64" ]; then \
        RUST_TARGET=x86_64-unknown-linux-gnu; \
    else \
        echo "Unsupported architecture: $TARGETARCH" && \
        exit 1; \
    fi && \
    cargo build --release --target=${RUST_TARGET} && \
    stat target/${RUST_TARGET}/release/main && \
    cp target/${RUST_TARGET}/release/main keipes-ai-mcp && \
    chmod +x keipes-ai-mcp && \
    ls -la /app/target/${RUST_TARGET}/release/ && \
    head -n 50 src/main.rs;

FROM debian:bookworm-slim

# Install PostgreSQL and required packages
RUN apt-get update && apt-get install -y \
    postgresql \
    postgresql-contrib \
    ca-certificates \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Setup PostgreSQL
USER postgres
RUN /etc/init.d/postgresql start && \
    psql --command "ALTER USER postgres PASSWORD 'postgres';" && \
    createdb keipes_mcp

USER root

WORKDIR /app
COPY --from=builder /app/keipes-ai-mcp ./keipes-ai-mcp

# Create startup script
RUN echo '#!/bin/bash\n\
LOG_FILE="/var/log/keipes-ai-mcp.log"\n\
exec > >(tee -a $LOG_FILE) 2>&1\n\
echo "Starting keipes-ai-mcp at $(date)"\n\
service postgresql start\n\
until pg_isready -h localhost -p 5432 -U postgres; do\n\
  echo "Waiting for PostgreSQL to be ready..."\n\
  sleep 1\n\
done\n\
echo "PostgreSQL is ready"\n\
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/keipes_mcp"\n\
echo "Starting MCP server..."\n\
exec ./keipes-ai-mcp' > /start.sh && \
    chmod +x /start.sh

EXPOSE 80

CMD ["/start.sh"]
