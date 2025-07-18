FROM rust:latest AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

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
COPY --from=builder /app/target/release/main ./keipes-ai-mcp

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
