# Audit Service

Version: 1.0.0
Last Updated: 2026-01-08

## Overview

AuditService handles logging of role changes to:
1. Ephemeral messages to the user
2. Audit log channel (if configured)

## Dependencies

- GuildConfigRepository
- Twilight HTTP Client

## Public Interface

```rust
impl AuditService {
    /// Log role addition
    pub async fn log_role_added(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
        role_name: &str,
        panel_name: &str,
    ) -> Result<(), ServiceError>;

    /// Log role removal
    pub async fn log_role_removed(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
        role_name: &str,
        panel_name: &str,
    ) -> Result<(), ServiceError>;

    /// Log multiple role changes (for sync)
    pub async fn log_role_sync(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        added: &[(Id<RoleMarker>, String)],  // (role_id, role_name)
        removed: &[(Id<RoleMarker>, String)],
        panel_name: &str,
    ) -> Result<(), ServiceError>;
}
```

## Behavior

### log_role_added / log_role_removed

1. Get guild config
2. If audit_channel_id is set:
   - Build audit embed
   - Send to audit channel
3. Return success (audit log failure should not block operation)

### Audit Embed Format

```
+------------------------------------------+
| Role Added                               |
+------------------------------------------+
| User: @username                          |
| Role: @rolename                          |
| Panel: Panel Name                        |
| Time: 2026-01-08 12:34:56 UTC           |
+------------------------------------------+
Color: Green (#57F287) for add, Red (#ED4245) for remove
```

### log_role_sync

1. Get guild config
2. If audit_channel_id is set:
   - Build summary embed
   - Send to audit channel
3. Return success

### Sync Audit Embed Format

```
+------------------------------------------+
| Roles Updated                            |
+------------------------------------------+
| User: @username                          |
| Panel: Panel Name                        |
|                                          |
| Added:                                   |
| - @role1                                 |
| - @role2                                 |
|                                          |
| Removed:                                 |
| - @role3                                 |
|                                          |
| Time: 2026-01-08 12:34:56 UTC           |
+------------------------------------------+
Color: Blurple (#5865F2)
```

## Error Handling

Audit logging MUST NOT fail the parent operation.
All errors should be logged but not propagated.

```rust
// In RoleService
if let Err(e) = self.audit.log_role_added(...).await {
    tracing::warn!("Failed to send audit log: {}", e);
}
// Continue with operation
```

## Configuration

Audit channel is configured per-guild via `/config audit-channel` command.
If not configured, audit logging is silently skipped.
