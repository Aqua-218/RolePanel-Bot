# Non-Functional Requirements

Version: 1.0.0
Last Updated: 2026-01-08

## NFR-001: Performance

### NFR-001-1: Interaction Response Time
- Bot MUST acknowledge interactions within 3 seconds (Discord requirement)
- Bot SHOULD respond to button clicks within 500ms
- Bot SHOULD complete role operations within 2 seconds

### NFR-001-2: Memory Usage
- Bot SHOULD use less than 128MB RAM under normal operation
- Bot MUST NOT have memory leaks

### NFR-001-3: Startup Time
- Bot SHOULD complete startup within 10 seconds
- Bot MUST be ready to receive events within 30 seconds

---

## NFR-002: Reliability

### NFR-002-1: Availability
- Bot MUST support graceful shutdown on SIGTERM
- Bot MUST reconnect automatically on gateway disconnect
- Bot MUST preserve session for resume when possible

### NFR-002-2: Data Integrity
- Bot MUST use database transactions for multi-step operations
- Bot MUST NOT lose panel data on crash
- Bot MUST handle database connection failures gracefully

### NFR-002-3: Error Recovery
- Bot MUST NOT crash on any user input
- Bot MUST log errors with sufficient context for debugging
- Bot MUST continue operating after non-fatal errors

---

## NFR-003: Scalability

### NFR-003-1: Single Shard Operation
- Bot MUST operate correctly with single shard (personal server use)
- Architecture SHOULD allow future multi-shard expansion

### NFR-003-2: Database Connections
- Bot MUST use connection pooling
- Pool size SHOULD be configurable via environment variable

---

## NFR-004: Security

### NFR-004-1: Token Security
- Bot MUST read Discord token from environment variable only
- Bot MUST NOT log or display token in any output

### NFR-004-2: Database Security
- Bot MUST read database URL from environment variable only
- Bot MUST use parameterized queries (no SQL injection)

### NFR-004-3: Permission Validation
- Bot MUST verify user has Manage Roles permission for panel management
- Bot MUST verify user has Administrator permission for configuration

---

## NFR-005: Maintainability

### NFR-005-1: Code Organization
- Code MUST follow Rust 2021 edition idioms
- Code MUST have zero clippy warnings (pedantic level)
- Code MUST be formatted with rustfmt

### NFR-005-2: Logging
- Bot MUST use structured logging (tracing crate)
- Bot MUST log at appropriate levels (error, warn, info, debug, trace)
- Log level MUST be configurable via environment variable

### NFR-005-3: Configuration
- All configuration MUST be via environment variables
- Bot MUST validate configuration at startup
- Bot MUST fail fast with clear error on invalid configuration

---

## NFR-006: Deployability

### NFR-006-1: Container Support
- Bot MUST run in Docker container
- Dockerfile MUST produce minimal image (multi-stage build)
- Image SHOULD be under 50MB

### NFR-006-2: Kubernetes Support
- Bot MUST expose health check endpoint (HTTP /health)
- Bot MUST handle SIGTERM for graceful shutdown
- Termination grace period SHOULD be 30 seconds

### NFR-006-3: Database Migration
- Bot MUST support automatic database migration on startup
- Migrations MUST be idempotent

---

## NFR-007: Observability

### NFR-007-1: Health Check
- Bot MUST expose HTTP endpoint for liveness check
- Bot MUST expose HTTP endpoint for readiness check
- Readiness MUST verify gateway connection and database connection

### NFR-007-2: Metrics (Future)
- Architecture SHOULD allow adding Prometheus metrics later
- Metrics endpoint MAY be added in future version

---

## NFR-008: Compatibility

### NFR-008-1: Rust Version
- Bot MUST compile with Rust 1.75.0 or later
- Bot SHOULD compile with latest stable Rust

### NFR-008-2: Database Version
- Bot MUST support PostgreSQL 14 or later
- Bot SHOULD support PostgreSQL 16
