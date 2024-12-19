# Use Ubuntu as base image
FROM ubuntu:22.04

# Combine all apt operations in a single RUN command and clean up in the same layer
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    postgresql \
    postgresql-client \
    postgresql-contrib \
    redis-server \
    supervisor && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    rm -rf /tmp/* /var/tmp/*

# Install Rust (with cleanup)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rm -rf /tmp/*
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Go (with cleanup)
RUN curl -OL https://golang.org/dl/go1.21.0.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.21.0.linux-amd64.tar.gz \
    && rm go1.21.0.linux-amd64.tar.gz \
    && rm -rf /tmp/*
ENV PATH="/usr/local/go/bin:${PATH}"

# Install Solana (with cleanup)
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)" \
    && rm -rf /tmp/*
ENV PATH="/root/.local/share/solana/install/active_release/bin:${PATH}"

# Install Anchor (with cleanup)
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force \
    && avm install latest \
    && avm use latest \
    && rm -rf /tmp/* \
    && rm -rf $HOME/.cargo/registry \
    && rm -rf $HOME/.cargo/git

# Set working directory
WORKDIR /app

# Create supervisor configuration
RUN mkdir -p /var/log/supervisor
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Initialize PostgreSQL data directory
RUN mkdir -p /var/run/postgresql && chown -R postgres:postgres /var/run/postgresql \
    && mkdir -p /var/lib/postgresql/data && chown -R postgres:postgres /var/lib/postgresql/data \
    && su - postgres -c "/usr/lib/postgresql/14/bin/initdb -D /var/lib/postgresql/data" \
    && echo "host all  all    0.0.0.0/0  md5" >> /var/lib/postgresql/data/pg_hba.conf \
    && echo "listen_addresses='*'" >> /var/lib/postgresql/data/postgresql.conf

# Copy only necessary project files
COPY . .

# Expose necessary ports
EXPOSE 8080 6379 5432

# Start services using supervisor
CMD ["/usr/bin/supervisord", "-n", "-c", "/etc/supervisor/conf.d/supervisord.conf"]