# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TRX Tracker is a high-performance, unified TRX/USDT transaction monitoring system built with Rust and React. It provides real-time blockchain scanning, multi-address querying, webhook notifications, and WebSocket streaming with a comprehensive admin dashboard.

## Development Commands

### Building and Running
```bash
# Build the project (debug mode)
cargo build

# Build for production (optimized)
cargo build --release

# Run the application
cargo run

# Run with custom config
cargo run -- --config config/development.toml
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check compilation without building
cargo check
```

### Database Setup
```bash
# Apply migrations (migrations are in migrations/ directory)
# The system will auto-apply migrations on startup, but you can run manually:
psql -U postgres -d trontracker -f migrations/001_initial.sql
```

### Docker Development
```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Rebuild after changes
docker-compose build && docker-compose up -d
```

## Architecture Overview

The codebase follows a unified, modular architecture:

### Core Modules (`src/core/`)
- **models.rs**: Shared data models (Transaction, Address, Webhook, etc.) with database serialization
- **database.rs**: PostgreSQL connection pooling and query execution
- **config.rs**: Hierarchical configuration with TOML files and environment overrides
- **tron_client.rs**: Blockchain interaction with multi-node failover

### Service Layer (`src/services/`)
- **scanner.rs**: Blockchain scanning with concurrent processing and retry logic
- **webhook.rs**: Webhook delivery with HMAC signing and exponential backoff
- **websocket.rs**: Real-time WebSocket connections with subscription management
- **auth.rs**: API key authentication and permission management
- **cache.rs**: Multi-layer Redis caching with TTL management

### API Layer (`src/api/`)
- **handlers/**: REST API endpoints organized by domain (transactions, addresses, webhooks, admin)
- **middleware.rs**: Authentication, rate limiting, and request logging
- **router.rs**: Axum route configuration with middleware chain

### Key Design Patterns

1. **Async-First**: All I/O operations use Tokio async runtime
2. **Error Propagation**: Result types with custom error enums using `thiserror`
3. **Connection Pooling**: Database and Redis connections are pooled for efficiency
4. **Graceful Degradation**: Services continue operating even if non-critical components fail
5. **Configuration Hierarchy**: Default → Local → Environment variables

### Database Schema

Tables:
- `transactions`: Core transaction data with comprehensive indexing
- `addresses`: Address metadata and statistics
- `webhooks`: Webhook configurations with delivery status
- `api_keys`: Authentication credentials with permissions
- `system_config`: Dynamic system configuration

Key indexes:
- Transaction hash (unique)
- Address + timestamp (for efficient querying)
- Block number (for scanning)
- Token transfers (for USDT tracking)

### Performance Considerations

- Batch processing for blockchain scanning (100 blocks at a time)
- Multi-layer caching: In-memory → Redis → Database
- Connection pooling with configurable limits
- Prepared statements for all database queries
- Concurrent webhook delivery with rate limiting

## Testing Approach

Tests are embedded in modules using `#[cfg(test)]` blocks. Key test areas:
- Service logic (scanner, webhook delivery)
- Database operations (models, queries)
- API endpoints (request/response validation)
- Configuration parsing and validation

Run specific test modules:
```bash
cargo test services::scanner
cargo test api::handlers::transaction
```

## Common Development Tasks

### Adding a New API Endpoint
1. Define handler in `src/api/handlers/`
2. Add route in `src/api/router.rs`
3. Update OpenAPI documentation if needed
4. Add corresponding tests

### Modifying Database Schema
1. Create new migration file in `migrations/`
2. Update models in `src/core/models.rs`
3. Run migration manually or restart application
4. Update relevant service logic

### Adding Configuration Options
1. Update `Config` struct in `src/core/config.rs`
2. Add default values in `config/default.toml`
3. Document new options in configuration section
4. Support environment variable override

### Debugging WebSocket Connections
1. Enable debug logging: `RUST_LOG=tron_tracker::services::websocket=debug`
2. Monitor active connections via admin API
3. Use wscat or similar for manual testing
4. Check Redis for subscription state

## Important Notes

- Always run `cargo fmt` before committing code
- Use `cargo clippy` to catch common issues
- Database migrations are applied automatically on startup
- Redis is required for caching and WebSocket state
- Admin UI is built separately and served from `admin-ui/dist`
- Configuration uses TOML format with environment overrides
- All times are stored in UTC in the database
- Transaction amounts are stored as strings to preserve precision