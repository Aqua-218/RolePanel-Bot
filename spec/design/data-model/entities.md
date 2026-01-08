# Panel Entity

Version: 1.0.0
Last Updated: 2026-01-08

## Definition

```rust
pub struct Panel {
    pub id: Uuid,
    pub guild_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub style: PanelStyle,
    pub color: i32,
    pub channel_id: Option<i64>,
    pub message_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PanelStyle {
    Button,
    SelectMenu,
}
```

## Fields

### id
- Type: UUID v4
- Generated on creation
- Used in custom_id for interactions

### guild_id
- Type: i64 (Discord snowflake)
- Discord guild where panel belongs
- Used for authorization checks

### name
- Type: String
- Max length: 100 characters
- Unique per guild
- Used for `/panel edit <name>` autocomplete

### description
- Type: Optional String
- Max length: 4096 characters (embed limit)
- Displayed in panel embed

### style
- Type: Enum
- Values: `button`, `select_menu`
- Determines rendering mode

### color
- Type: i32
- Embed sidebar color
- Stored as decimal (e.g., 0x5865F2 = 5793266)

### channel_id
- Type: Optional i64
- Set when panel is posted
- NULL for draft panels

### message_id
- Type: Optional i64
- Set when panel is posted
- Used for updating existing panel

### created_at / updated_at
- Type: DateTime<Utc>
- Automatic timestamps

## Invariants

1. `name` MUST be unique within a guild
2. If `message_id` is set, `channel_id` MUST also be set
3. `color` MUST be in range 0-16777215 (0x000000-0xFFFFFF)

## Custom ID Format

Panel-related custom IDs follow this format:

```
panel:{panel_id}:{action}
```

Examples:
- `panel:550e8400-e29b-41d4-a716-446655440000:add_role`
- `panel:550e8400-e29b-41d4-a716-446655440000:preview`
- `panel:550e8400-e29b-41d4-a716-446655440000:post`
