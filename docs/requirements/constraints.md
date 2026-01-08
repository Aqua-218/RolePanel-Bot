# Constraints

Version: 1.0.0
Last Updated: 2026-01-08

## Technical Constraints

### CON-001: Language
- Implementation language: Rust
- No C language usage
- No C-dependent libraries (unless absolutely necessary and explicitly approved)
- No FFI calls to C code

### CON-002: License
- All dependencies MUST use permissive licenses
- Allowed: MIT, Apache 2.0, BSD, ISC, Zlib, CC0, Unlicense
- Prohibited: GPL, LGPL, AGPL, MPL (copyleft licenses)

### CON-003: Discord API Limits
| Resource | Limit |
|----------|-------|
| Buttons per row | 5 |
| Button rows per message | 5 |
| Total buttons per message | 25 |
| Select menu options | 25 |
| Select menus per message | 5 |
| Custom ID length | 100 characters |
| Embed title | 256 characters |
| Embed description | 4096 characters |
| Modal text input | 4000 characters |
| Interaction response time | 3 seconds |

### CON-004: Database
- Database: PostgreSQL (required)
- ORM/Query: SQLx (compile-time checked queries)
- Connection pooling: Required

### CON-005: Runtime
- Async runtime: Tokio
- Single-threaded runtime is acceptable for personal use

## Operational Constraints

### CON-006: Deployment Environment
- Target: Kubernetes
- Container runtime: Docker
- Single replica (no horizontal scaling required)

### CON-007: Resource Limits
- Memory request: 64MB
- Memory limit: 128MB
- CPU request: 50m
- CPU limit: 200m

## Development Constraints

### CON-008: Code Style
- No emoji in code, comments, or commit messages
- Comments in English
- Documentation in Japanese (user-facing)
- Commit messages in English

### CON-009: Testing
- Target test coverage: 95%+
- Integration tests for database operations
- Unit tests for business logic

### CON-010: Dependencies
- Minimize dependencies
- Each dependency must be justified
- Audit dependencies for security issues
