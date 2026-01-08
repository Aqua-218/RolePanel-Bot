# Role Service

Version: 1.0.0
Last Updated: 2026-01-08

## Overview

RoleService handles role assignment and removal for users interacting with panels.

## Dependencies

- PanelRoleRepository
- Twilight HTTP Client
- AuditService

## Public Interface

```rust
impl RoleService {
    /// Toggle role for user (add if not present, remove if present)
    /// Used for button-style panels
    /// Returns: (added: bool, role_id)
    /// Errors: ERR-003 (not found), ERR-004 (discord error), ERR-006 (permission)
    pub async fn toggle_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
    ) -> Result<(bool, Id<RoleMarker>), ServiceError>;

    /// Sync roles for user based on selection
    /// Used for select-menu-style panels
    /// Returns: (added: Vec<role_id>, removed: Vec<role_id>)
    /// Errors: ERR-003 (not found), ERR-004 (discord error), ERR-006 (permission)
    pub async fn sync_roles(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        panel_id: Uuid,
        selected_role_ids: Vec<Id<RoleMarker>>,
    ) -> Result<RoleSyncResult, ServiceError>;
}

pub struct RoleSyncResult {
    pub added: Vec<Id<RoleMarker>>,
    pub removed: Vec<Id<RoleMarker>>,
}
```

## Behavior

### toggle_role

1. Get panel_role by panel_id and role_id
2. Fetch guild member
3. Check if member has role:
   - If yes: remove role
   - If no: add role
4. Log action via AuditService
5. Return (added, role_id)

### sync_roles

1. Get all panel_roles for panel
2. Fetch guild member
3. Determine member's current panel roles
4. Calculate diff:
   - To add: selected but not possessed
   - To remove: possessed but not selected (panel roles only)
5. Apply changes
6. Log actions via AuditService
7. Return RoleSyncResult

## Permission Validation

Before any role operation:

```rust
async fn validate_role_operation(
    &self,
    guild_id: Id<GuildMarker>,
    role_id: Id<RoleMarker>,
) -> Result<(), ServiceError> {
    let role = self.http.role(guild_id, role_id).await?;
    let bot_member = self.http.guild_member(guild_id, self.bot_id).await?;
    let bot_highest_role = self.get_highest_role_position(&bot_member);
    
    if role.position >= bot_highest_role {
        return Err(ServiceError::Permission(
            "Role is higher than or equal to bot's highest role".into()
        ));
    }
    
    if role.managed {
        return Err(ServiceError::Permission(
            "Cannot assign managed role".into()
        ));
    }
    
    Ok(())
}
```

## Error Handling

| Code | Name | Description |
|------|------|-------------|
| ERR-003 | NotFound | Panel role not found |
| ERR-004 | DiscordError | Discord API error |
| ERR-006 | Permission | Bot lacks permission to manage role |
