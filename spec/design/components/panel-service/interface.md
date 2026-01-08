# Panel Service

Version: 1.0.0
Last Updated: 2026-01-08

## Overview

PanelService handles all panel-related business logic including creation, modification,
deletion, and posting of role panels.

## Dependencies

- PanelRepository
- PanelRoleRepository
- Twilight HTTP Client

## Public Interface

```rust
impl PanelService {
    /// Create a new panel
    /// Returns: Created panel
    /// Errors: ERR-001 (name exists), ERR-002 (db error)
    pub async fn create_panel(
        &self,
        guild_id: Id<GuildMarker>,
        name: String,
        description: Option<String>,
    ) -> Result<Panel, ServiceError>;

    /// Get panel by ID
    /// Errors: ERR-003 (not found), ERR-002 (db error)
    pub async fn get_panel(
        &self,
        panel_id: Uuid,
    ) -> Result<Panel, ServiceError>;

    /// Get panel with roles
    /// Errors: ERR-003 (not found), ERR-002 (db error)
    pub async fn get_panel_with_roles(
        &self,
        panel_id: Uuid,
    ) -> Result<(Panel, Vec<PanelRole>), ServiceError>;

    /// List panels for guild
    /// Errors: ERR-002 (db error)
    pub async fn list_panels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<Panel>, ServiceError>;

    /// Update panel
    /// Errors: ERR-003 (not found), ERR-001 (name exists), ERR-002 (db error)
    pub async fn update_panel(
        &self,
        panel_id: Uuid,
        update: PanelUpdate,
    ) -> Result<Panel, ServiceError>;

    /// Delete panel (and posted message if exists)
    /// Errors: ERR-003 (not found), ERR-002 (db error), ERR-004 (discord error)
    pub async fn delete_panel(
        &self,
        panel_id: Uuid,
    ) -> Result<(), ServiceError>;

    /// Add role to panel
    /// Errors: ERR-003 (not found), ERR-005 (limit exceeded), ERR-002 (db error)
    pub async fn add_role(
        &self,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
        label: String,
        emoji: Option<String>,
        description: Option<String>,
    ) -> Result<PanelRole, ServiceError>;

    /// Remove role from panel
    /// Errors: ERR-003 (not found), ERR-002 (db error)
    pub async fn remove_role(
        &self,
        panel_id: Uuid,
        role_id: Id<RoleMarker>,
    ) -> Result<(), ServiceError>;

    /// Post panel to channel (or update if already posted)
    /// Errors: ERR-003 (not found), ERR-004 (discord error), ERR-002 (db error)
    pub async fn post_panel(
        &self,
        panel_id: Uuid,
        channel_id: Id<ChannelMarker>,
    ) -> Result<(), ServiceError>;
}
```

## Update Structure

```rust
pub struct PanelUpdate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,  // None = no change, Some(None) = clear
    pub style: Option<PanelStyle>,
    pub color: Option<i32>,
}
```

## Behavior

### create_panel

1. Validate name (non-empty, max 100 chars)
2. Check name uniqueness within guild
3. Create panel record with default style (Button) and color (Blurple)
4. Return created panel

### get_panel_with_roles

1. Query panel by ID
2. Query associated panel_roles ordered by position
3. Return tuple of panel and roles

### add_role

1. Verify panel exists
2. Count existing roles
3. If count >= 25, return ERR-005
4. Calculate next position
5. Insert panel_role record
6. If panel is posted, update message
7. Return created panel_role

### post_panel

1. Get panel with roles
2. Build embed (title, description, color, role list)
3. Build components based on style
4. If message_id exists:
   - Update existing message
5. Else:
   - Create new message
   - Store channel_id and message_id
6. Return success

## Error Handling

| Code | Name | Description |
|------|------|-------------|
| ERR-001 | NameExists | Panel name already exists in guild |
| ERR-002 | DatabaseError | Database operation failed |
| ERR-003 | NotFound | Panel or role not found |
| ERR-004 | DiscordError | Discord API error |
| ERR-005 | LimitExceeded | Role limit (25) exceeded |
