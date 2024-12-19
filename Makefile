.PHONY: build run test clean docker-build docker-run docker-stop docker-attach

# Docker configuration
DOCKER_IMAGE = dex-platform
DOCKER_CONTAINER = dex-platform-dev
HOST_PORT = 8080
CONTAINER_PORT = 8080

# Project directories
RUST_DIR = backend/orderbook
GO_DIR = backend/api
SOLANA_DIR = contracts/solana

# Build commands
build:
	@echo "Building all components..."
	@cd $(RUST_DIR) && cargo build
	@cd $(GO_DIR) && go build
	@cd $(SOLANA_DIR) && anchor build

test:
	@echo "Running tests..."
	@cd $(RUST_DIR) && cargo test
	@cd $(GO_DIR) && go test ./...
	@cd $(SOLANA_DIR) && anchor test

clean:
	@echo "Cleaning build artifacts..."
	@cd $(RUST_DIR) && cargo clean
	@cd $(GO_DIR) && go clean
	@rm -rf target/

# Docker commands
docker-build:
	docker build -t $(DOCKER_IMAGE) .

docker-run:
	@echo "Checking for existing container..."
ifeq ($(OS),Windows_NT)
	@docker stop $(DOCKER_CONTAINER) 2>NUL || exit 0
	@docker rm $(DOCKER_CONTAINER) 2>NUL || exit 0
else
	@docker stop $(DOCKER_CONTAINER) 2>/dev/null || true
	@docker rm $(DOCKER_CONTAINER) 2>/dev/null || true
endif
	@docker run -d \
		--name $(DOCKER_CONTAINER) \
		-p $(HOST_PORT):$(CONTAINER_PORT) \
		-v "$(CURDIR):/app" \
		-w /app \
		$(DOCKER_IMAGE)
	@timeout /t 5 /nobreak >nul
	@docker ps | findstr $(DOCKER_CONTAINER) || (docker logs $(DOCKER_CONTAINER) && exit 1)
	@echo "Container started successfully"

docker-restart: docker-stop docker-run
	@echo "Container restarted successfully"

docker-stop:
	@echo "Stopping container..."
	@docker stop $(DOCKER_CONTAINER) >/dev/null 2>&1 || true
	@docker rm $(DOCKER_CONTAINER) >/dev/null 2>&1 || true
	@echo "Container stopped"

docker-attach:
	docker exec -it $(DOCKER_CONTAINER) /bin/bash

# Development helpers
dev: docker-build docker-run
	@echo "Development environment is ready"
	@echo "Use 'make docker-attach' to connect to the container"

stop: docker-stop
	@echo "Development environment stopped"