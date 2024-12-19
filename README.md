# DEX Platform

A high-performance decentralized exchange platform built with Rust, Go, and Solana.

## Features

- High-performance order matching engine
- Real-time market data streaming
- On-chain settlement via Solana
- Advanced order types support
- Professional-grade API
- Comprehensive monitoring and analytics

## Prerequisites

- Docker and Docker Compose
- Rust (1.75.0 or later)
- Go (1.21.6 or later)
- Node.js (18.x or later)
- Solana CLI tools
- Make

## Quick Start

1. Clone the repository:
```bash
git clone https://github.com/yourusername/dex-platform
cd dex-platform
```

2. Set up the development environment:
```bash
# Initialize the project
make init

# Build the Docker containers
make docker-build

# Start the services
make docker-run
```

3. Verify the setup:
```bash
# Check service health
./scripts/healthcheck.sh
```

## Development

### Project Structure

```
.
├── api/                 # Go API service
│   ├── cmd/            # Entry points
│   ├── internal/       # Private application code
│   └── pkg/            # Public library code
├── core/               # Rust core service
│   └── src/            # Core engine code
├── programs/           # Solana programs
│   ├── dex/           # Main DEX program
│   └── oracle/         # Price oracle program
└── monitoring/         # Monitoring setup
```

### Available Make Commands

- `make build` - Build all components
- `make test` - Run tests
- `make run` - Run the development environment
- `make clean` - Clean build artifacts
- `make lint` - Run linters
- `make fmt` - Format code

### Development Workflow

1. Create a new feature branch:
```bash
git checkout -b feature/your-feature
```

2. Make your changes and ensure tests pass:
```bash
make test
```

3. Format and lint your code:
```bash
make fmt
make lint
```

4. Submit a pull request

## API Documentation

The API documentation is available at:
- REST API: `http://localhost:8080/swagger/index.html`
- WebSocket API: `ws://localhost:8080/ws`

## Monitoring

Access the monitoring dashboards at:
- Grafana: `http://localhost:3000`
- Prometheus: `http://localhost:9090`

## Architecture

See [ARCHITECTURE.md](docs/architecture/ARCHITECTURE.md) for detailed system design documentation.

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Security

For security concerns, please email security@yourdomain.com.

## Support

For support questions, please use [GitHub Issues](https://github.com/yourusername/dex-platform/issues). 