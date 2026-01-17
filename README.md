# 5G Service Communication Proxy (SCP)

A 5G Service Communication Proxy implementation in Rust following 3GPP specifications.

## Features

- Service-based architecture proxy and routing
- NRF integration for service discovery
- HTTP/2 support
- TLS/mTLS support
- OAuth2 authentication
- Request forwarding and routing
- Load balancing
- NF profile caching

## Configuration

Configuration is managed through environment variables. See `.env.example` for available options.

Key configuration:
- `SCP_HOST`: Bind address (default: 0.0.0.0)
- `SCP_PORT`: Listen port (default: 7777)
- `MONGODB_URI`: MongoDB connection string
- `NRF_URI`: Network Repository Function URI for service discovery
- `CACHE_TTL_SECONDS`: NF profile cache TTL (default: 300)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Docker

```bash
docker build -t scp .
docker run -p 7777:7777 --env-file .env scp
```

## API Endpoints

- `GET /health` - Health check endpoint
- `GET /status` - Service status and version information

## Architecture

- `src/clients/` - External service clients (NRF)
- `src/handlers/` - HTTP request handlers
- `src/services/` - Business logic and routing services
- `src/middleware/` - Authentication and validation middleware
- `src/types/` - Type definitions and models
- `src/config.rs` - Configuration management
- `src/db.rs` - Database initialization

## Standards Compliance

- 3GPP TS 29.500 series (5G System Architecture)
- 3GPP TS 29.549 (Service Communication Proxy services)
