# Data Model Overview

Version: 1.0.0
Last Updated: 2026-01-08

## Entity Relationship Diagram

```
+------------------+       +-------------------+
|   guild_configs  |       |      panels       |
+------------------+       +-------------------+
| guild_id (PK)    |       | id (PK)           |
| audit_channel_id |       | guild_id (FK)     |----+
| created_at       |       | name              |    |
| updated_at       |       | description       |    |
+------------------+       | style             |    |
                           | color             |    |
                           | channel_id        |    |
                           | message_id        |    |
                           | created_at        |    |
                           | updated_at        |    |
                           +-------------------+    |
                                    |               |
                                    | 1:N           |
                                    |               |
                           +--------v----------+    |
                           |   panel_roles     |    |
                           +-------------------+    |
                           | id (PK)           |    |
                           | panel_id (FK)     |----+
                           | role_id           |
                           | label             |
                           | emoji             |
                           | description       |
                           | position          |
                           | created_at        |
                           +-------------------+
```

## Tables

### guild_configs
Guild-level configuration storage.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| guild_id | BIGINT | No | Primary key, Discord guild ID |
| audit_channel_id | BIGINT | Yes | Audit log channel ID |
| created_at | TIMESTAMPTZ | No | Record creation time |
| updated_at | TIMESTAMPTZ | No | Last update time |

### panels
Role panel definitions.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | UUID | No | Primary key |
| guild_id | BIGINT | No | Discord guild ID |
| name | VARCHAR(100) | No | Panel name (unique per guild) |
| description | TEXT | Yes | Panel description |
| style | VARCHAR(20) | No | 'button' or 'select_menu' |
| color | INTEGER | No | Embed color (decimal) |
| channel_id | BIGINT | Yes | Posted channel ID |
| message_id | BIGINT | Yes | Posted message ID |
| created_at | TIMESTAMPTZ | No | Record creation time |
| updated_at | TIMESTAMPTZ | No | Last update time |

### panel_roles
Roles assigned to panels.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | UUID | No | Primary key |
| panel_id | UUID | No | Foreign key to panels |
| role_id | BIGINT | No | Discord role ID |
| label | VARCHAR(80) | No | Display label |
| emoji | VARCHAR(100) | Yes | Emoji (unicode or custom) |
| description | VARCHAR(100) | Yes | Description (select menu only) |
| position | INTEGER | No | Display order |
| created_at | TIMESTAMPTZ | No | Record creation time |

## Indexes

```sql
-- panels
CREATE INDEX idx_panels_guild_id ON panels(guild_id);
CREATE UNIQUE INDEX idx_panels_guild_name ON panels(guild_id, name);
CREATE INDEX idx_panels_message_id ON panels(message_id) WHERE message_id IS NOT NULL;

-- panel_roles
CREATE INDEX idx_panel_roles_panel_id ON panel_roles(panel_id);
CREATE INDEX idx_panel_roles_role_id ON panel_roles(role_id);
```

## Constraints

```sql
-- panels
ALTER TABLE panels ADD CONSTRAINT chk_panels_style 
  CHECK (style IN ('button', 'select_menu'));

-- panel_roles
ALTER TABLE panel_roles ADD CONSTRAINT fk_panel_roles_panel
  FOREIGN KEY (panel_id) REFERENCES panels(id) ON DELETE CASCADE;
```
