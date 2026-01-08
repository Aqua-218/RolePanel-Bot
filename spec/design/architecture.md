# Architecture

Version: 1.0.0
Last Updated: 2026-01-08

## Overview

Discord Role Panel Bot is a single-binary Rust application that connects to Discord Gateway
and PostgreSQL database. It handles slash commands and component interactions to manage
role panels.

## Architecture Diagram

```
                                    +------------------+
                                    |   Discord API    |
                                    +--------+---------+
                                             |
                              +--------------+--------------+
                              |                             |
                     +--------v--------+          +---------v--------+
                     | Gateway (WSS)   |          | HTTP API (REST)  |
                     +--------+--------+          +---------+--------+
                              |                             |
                              +-------------+---------------+
                                            |
                    +-----------------------v-----------------------+
                    |                  Bot Process                  |
                    |  +------------------------------------------+ |
                    |  |              Event Router                | |
                    |  +----+-------------+-------------+---------+ |
                    |       |             |             |           |
                    |  +----v----+  +-----v-----+  +----v----+     |
                    |  | Command |  |Interaction|  | Gateway |     |
                    |  | Handler |  |  Handler  |  | Events  |     |
                    |  +----+----+  +-----+-----+  +----+----+     |
                    |       |             |             |           |
                    |  +----v-------------v-------------v----+     |
                    |  |            Service Layer            |     |
                    |  |  +-------+  +-------+  +-------+   |     |
                    |  |  | Panel |  | Role  |  | Audit |   |     |
                    |  |  |Service|  |Service|  |Service|   |     |
                    |  |  +---+---+  +---+---+  +---+---+   |     |
                    |  +------+----------+----------+-------+     |
                    |         |          |          |              |
                    |  +------v----------v----------v-------+     |
                    |  |          Repository Layer          |     |
                    |  +----------------+-------------------+     |
                    |                   |                          |
                    +-------------------|--------------------------+
                                        |
                              +---------v---------+
                              |    PostgreSQL     |
                              +-------------------+
                              
                    +-------------------+
                    |  Health Server    |
                    |  (HTTP :8080)     |
                    +-------------------+
```

## Layers

### Gateway Layer
- **Twilight Gateway**: WebSocket connection to Discord
- Single shard operation
- Handles reconnection and session resume
- Receives all Discord events

### Event Router
- Routes incoming events to appropriate handlers
- Filters relevant events (InteractionCreate, Ready, etc.)
- Provides error boundary for handlers

### Handler Layer
- **Command Handler**: Processes slash commands
- **Interaction Handler**: Processes button clicks, select menus, modals
- **Gateway Events**: Handles Ready, GuildCreate, etc.

### Service Layer
- **PanelService**: Panel CRUD operations, posting logic
- **RoleService**: Role assignment/removal logic
- **AuditService**: Logging to audit channel

### Repository Layer
- Database access abstraction
- SQLx queries with compile-time checking
- Transaction management

### Health Server
- Separate HTTP server on port 8080
- `/health/live` - Liveness probe
- `/health/ready` - Readiness probe (checks gateway + DB)

## Component Interaction

### Panel Creation Flow

```
User                Bot                    Database
 |                   |                        |
 |--/panel create--->|                        |
 |                   |                        |
 |<--Modal-----------|                        |
 |                   |                        |
 |--Submit---------->|                        |
 |                   |---INSERT panel-------->|
 |                   |<--OK-------------------|
 |<--Edit Interface--|                        |
 |                   |                        |
```

### Role Assignment Flow

```
User                Bot                    Discord API    Database
 |                   |                        |              |
 |--Button Click---->|                        |              |
 |                   |--Defer Ephemeral------>|              |
 |                   |---SELECT panel_role----|------------->|
 |                   |<--role_id--------------|--------------|
 |                   |--Add/Remove Role------>|              |
 |                   |<--OK-------------------|              |
 |<--Ephemeral Msg---|                        |              |
 |                   |                        |              |
 |                   |--Audit Log Embed------>|              |
 |                   |                        |              |
```

## Module Structure

```
src/
  main.rs                 # Entry point, initialization
  config.rs               # Configuration from environment
  error.rs                # Error types
  
  gateway/
    mod.rs
    event_loop.rs         # Main event loop
    router.rs             # Event routing
    
  handler/
    mod.rs
    command/
      mod.rs
      panel.rs            # /panel commands
      config.rs           # /config commands
    interaction/
      mod.rs
      panel_edit.rs       # Edit interface interactions
      role_select.rs      # Role assignment interactions
      
  service/
    mod.rs
    panel.rs              # Panel business logic
    role.rs               # Role assignment logic
    audit.rs              # Audit logging
    
  repository/
    mod.rs
    panel.rs              # Panel DB operations
    panel_role.rs         # PanelRole DB operations
    guild_config.rs       # GuildConfig DB operations
    
  model/
    mod.rs
    panel.rs              # Panel entity
    panel_role.rs         # PanelRole entity
    guild_config.rs       # GuildConfig entity
    
  discord/
    mod.rs
    embed.rs              # Embed builders
    component.rs          # Component builders
    modal.rs              # Modal builders
    
  health/
    mod.rs
    server.rs             # Health check HTTP server
```

## Dependencies

| Crate | Purpose | License |
|-------|---------|---------|
| twilight-gateway | Discord Gateway connection | ISC |
| twilight-http | Discord HTTP API | ISC |
| twilight-model | Discord data types | ISC |
| twilight-util | Utilities (builders) | ISC |
| sqlx | Database access | MIT/Apache-2.0 |
| tokio | Async runtime | MIT |
| tracing | Structured logging | MIT |
| tracing-subscriber | Log output | MIT |
| serde | Serialization | MIT/Apache-2.0 |
| serde_json | JSON support | MIT/Apache-2.0 |
| dotenvy | .env file loading | MIT |
| hyper | HTTP server (health) | MIT |

## Configuration

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| DISCORD_TOKEN | Bot token | Yes | - |
| DATABASE_URL | PostgreSQL connection | Yes | - |
| RUST_LOG | Log level | No | info |
| HEALTH_PORT | Health server port | No | 8080 |
| DATABASE_MAX_CONNECTIONS | Pool size | No | 5 |
