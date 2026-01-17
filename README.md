# 5G Service Communication Proxy (SCP)

A 5G Service Communication Proxy implementation in Rust following 3GPP specifications.

## Configuration

Configuration is managed through environment variables. See `.env.example` for available options.

Key configuration:
- `SCP_HOST`: Bind address (default: 0.0.0.0)
- `SCP_PORT`: Listen port (default: 7777)
- `MONGODB_URI`: MongoDB connection string
- `NRF_URI`: Network Repository Function URI for service discovery
- `CACHE_TTL_SECONDS`: NF profile cache TTL (default: 300)

## Standards Compliance

- 3GPP TS 29.500 series (5G System Architecture)
- 3GPP TS 29.549 (Service Communication Proxy services)
