# Security Design

Version: 1.0.0
Last Updated: 2026-01-08

## Authentication

### Discord Bot Token
- Token MUST be provided via `DISCORD_TOKEN` environment variable
- Token MUST NOT be logged at any log level
- Token MUST NOT be included in error messages

### Database Credentials
- Connection string MUST be provided via `DATABASE_URL` environment variable
- Connection string MUST NOT be logged
- Connection string MUST NOT be included in error messages

## Authorization

### Permission Matrix

| Action | Required Permission |
|--------|---------------------|
| Create panel | Manage Roles |
| Edit panel | Manage Roles |
| Delete panel | Manage Roles |
| Post panel | Manage Roles + Send Messages (in target channel) |
| Assign role (user) | None (any user can use panels) |
| Configure audit channel | Administrator |

### Permission Checks

```rust
// Panel management check
fn can_manage_panels(member: &Member) -> bool {
    member.permissions.contains(Permissions::MANAGE_ROLES)
}

// Configuration check
fn can_configure(member: &Member) -> bool {
    member.permissions.contains(Permissions::ADMINISTRATOR)
}
```

### Role Assignment Validation

Before assigning a role, bot MUST verify:

1. Bot has `MANAGE_ROLES` permission in guild
2. Target role is below bot's highest role in hierarchy
3. Target role is not @everyone
4. Target role is not managed (bot role, integration role, etc.)

```rust
fn can_assign_role(bot_member: &Member, target_role: &Role) -> bool {
    let bot_highest = get_highest_role_position(bot_member);
    
    target_role.position < bot_highest
        && !target_role.managed
        && target_role.id != guild.id  // @everyone has same ID as guild
}
```

## Input Validation

### Panel Name
- Max length: 100 characters
- Allowed characters: alphanumeric, spaces, hyphens, underscores
- Trimmed of leading/trailing whitespace
- Must not be empty after trimming

### Panel Description
- Max length: 4096 characters
- HTML/Markdown sanitization not needed (Discord handles this)

### Role Label
- Max length: 80 characters
- Must not be empty

### Emoji
- Must be valid Unicode emoji or Discord custom emoji format
- Custom emoji format: `<:name:id>` or `<a:name:id>` (animated)

### Color
- Must be valid hex color or predefined color name
- Range: 0x000000 - 0xFFFFFF

## SQL Injection Prevention

All database queries MUST use parameterized queries via SQLx:

```rust
// Correct
sqlx::query!("SELECT * FROM panels WHERE guild_id = $1", guild_id)

// Forbidden - never do this
format!("SELECT * FROM panels WHERE guild_id = {}", guild_id)
```

SQLx compile-time checking prevents most SQL injection vulnerabilities.

## Rate Limiting

### Discord Rate Limits
- Twilight HTTP client handles rate limits automatically
- Bot MUST NOT implement custom rate limit bypass

### Abuse Prevention
- No additional rate limiting implemented (Discord's limits are sufficient)
- If abuse becomes an issue, consider adding per-user cooldowns

## Logging Security

### What to Log
- User actions (command usage, role changes)
- Errors with context
- Startup/shutdown events

### What NOT to Log
- Discord token
- Database connection string
- Full message content (unless necessary for debugging)
- Personal user data beyond Discord ID

### Log Sanitization

```rust
// Ensure secrets are never logged
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("discord_token", &"[REDACTED]")
            .field("database_url", &"[REDACTED]")
            .field("health_port", &self.health_port)
            .finish()
    }
}
```
