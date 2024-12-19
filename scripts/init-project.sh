#!/bin/bash

# Create project directory structure
mkdir -p api/{cmd,internal,pkg} \
       core/src \
       programs/{dex,oracle}/src \
       monitoring/{grafana,prometheus} \
       docs/{architecture,api} \
       scripts

# Initialize Go API project
cd api
go mod init github.com/yourusername/dex-platform
go mod tidy

# Create main.go
cat > cmd/main.go << 'EOL'
package main

import (
    "log"
    "github.com/yourusername/dex-platform/internal/app"
)

func main() {
    if err := app.Run(); err != nil {
        log.Fatal(err)
    }
}
EOL

# Create app.go
mkdir -p internal/app
cat > internal/app/app.go << 'EOL'
package app

import (
    "context"
    "net/http"
    "os"
    "os/signal"
    "syscall"
    "time"

    "github.com/gin-gonic/gin"
)

func Run() error {
    router := gin.Default()
    
    // Basic health check
    router.GET("/health", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{"status": "ok"})
    })

    srv := &http.Server{
        Addr:    ":8080",
        Handler: router,
    }

    // Graceful shutdown
    go func() {
        quit := make(chan os.Signal, 1)
        signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
        <-quit

        ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
        defer cancel()
        if err := srv.Shutdown(ctx); err != nil {
            os.Exit(1)
        }
    }()

    return srv.ListenAndServe()
}
EOL

cd ..

# Initialize Rust core project
cd core
cargo init --bin
cat > Cargo.toml << 'EOL'
[package]
name = "dex-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.28", features = ["full"] }
axum = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
redis = { version = "0.23", features = ["tokio-comp"] }
anyhow = "1.0"
thiserror = "1.0"
EOL

# Create main.rs with basic setup
cat > src/main.rs << 'EOL'
use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check));

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 9090));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
EOL
cd ..

# Initialize Solana program
cd programs/dex
cargo init --lib
cat > Cargo.toml << 'EOL'
[package]
name = "dex-program"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-program = "1.16"
anchor-lang = "0.28.0"
EOL

# Create lib.rs with basic Solana program
cat > src/lib.rs << 'EOL'
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}
EOL
cd ../..

# Create .env file
cat > .env << 'EOL'
# API Configuration
API_PORT=8080
API_HOST=0.0.0.0

# Database Configuration
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_DB=dex
POSTGRES_HOST=postgres
POSTGRES_PORT=5432

# Redis Configuration
REDIS_HOST=redis
REDIS_PORT=6379

# Monitoring
PROMETHEUS_PORT=9090
GRAFANA_PORT=3000

# JWT Configuration
JWT_SECRET=your-secret-key
JWT_EXPIRY=24h
EOL

# Create .gitignore
cat > .gitignore << 'EOL'
# Rust
/target/
**/*.rs.bk
Cargo.lock

# Go
/api/vendor/
*.exe
*.exe~
*.dll
*.so
*.dylib
*.test
*.out
go.sum

# Environment
.env
.env.local
.env.*.local

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Build
/dist/
/build/
/bin/

# Dependencies
/node_modules/

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Docker
.docker/
EOL

# Initialize monitoring
cat > monitoring/prometheus/prometheus.yml << 'EOL'
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'dex-core'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'

  - job_name: 'dex-api'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'

  - job_name: 'redis'
    static_configs:
      - targets: ['redis:6379']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres:5432']
EOL

echo "Project initialized successfully!" 