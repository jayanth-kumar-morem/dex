#!/bin/bash

# Create necessary directories
mkdir -p api/src core/src programs/src

# Initialize Rust projects
cd core
cargo init --bin
cd ../programs
cargo init --lib
cd ..

# Initialize Go project
cd api
go mod init github.com/yourusername/dex-platform
cd ..

# Create .env file
cat > .env << EOL
RUST_LOG=debug
POSTGRES_URL=postgres://postgres:postgres@postgres:5432/dex
REDIS_URL=redis://redis:6379
EOL

# Create .gitignore
cat > .gitignore << EOL
/target
**/*.rs.bk
*.pem
*.env
.DS_Store
/node_modules
/dist
Cargo.lock
go.sum
EOL

# Initialize core Rust project
cat > core/Cargo.toml << EOL
[package]
name = "dex-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
futures = "0.3"
EOL

# Initialize Solana program
cat > programs/Cargo.toml << EOL
[package]
name = "dex-program"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-program = "1.16"
anchor-lang = "0.28.0"
EOL

# Initialize Go API
cat > api/main.go << EOL
package main

import (
    "log"
    "net/http"
    "github.com/gin-gonic/gin"
)

func main() {
    r := gin.Default()
    r.GET("/health", func(c *gin.Context) {
        c.JSON(http.StatusOK, gin.H{
            "status": "ok",
        })
    })
    log.Fatal(r.Run(":8080"))
}
EOL

# Initialize Go dependencies
cd api
go get github.com/gin-gonic/gin
go get gorm.io/gorm
go get gorm.io/driver/postgres
cd ..

echo "Project initialized successfully!" 