# /panel edit

Version: 1.0.0
Last Updated: 2026-01-08

## Signature

```
/panel edit <name>
```

## Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| name | String | Yes | Panel name (autocomplete enabled) |

## Permission

User MUST have `Manage Roles` permission.

## Autocomplete

Bot MUST provide autocomplete for `name` parameter:
- Query panels matching input prefix
- Return up to 25 matches
- Format: `{panel_name}`

## Behavior

1. Find panel by name in current guild
2. Display edit interface (ephemeral)

## Response

### Success
Ephemeral message with edit interface.

### Errors

| Condition | Response |
|-----------|----------|
| Panel not found | "Panel not found." |
| Missing permission | "You need Manage Roles permission." |
| Internal error | "An error occurred. Please try again." |

## Edit Interface Specification

### Message Content

```
+------------------------------------------+
| Panel: {name}                            |
+------------------------------------------+
| {description or "No description"}        |
|                                          |
| Style: Button / Select Menu              |
| Color: [=====] (color preview)           |
|                                          |
| Roles ({count}/25):                      |
| - :emoji: Label (@role)                  |
| - :emoji: Label (@role)                  |
| (or "No roles added")                    |
|                                          |
| Status: Draft / Posted in #channel       |
+------------------------------------------+
```

### Components

#### Row 1
| Component | Type | Style | Custom ID |
|-----------|------|-------|-----------|
| Add Role | Button | Primary | `panel:{id}:add_role` |
| Remove Role | Button | Secondary | `panel:{id}:remove_role` |
| Style | Button | Secondary | `panel:{id}:style` |

#### Row 2
| Component | Type | Style | Custom ID |
|-----------|------|-------|-----------|
| Color | Button | Secondary | `panel:{id}:color` |
| Preview | Button | Secondary | `panel:{id}:preview` |
| Post | Button | Success | `panel:{id}:post` |

#### Row 3
| Component | Type | Style | Custom ID |
|-----------|------|-------|-----------|
| Delete | Button | Danger | `panel:{id}:delete` |

## Interaction Handlers

### Add Role (`panel:{id}:add_role`)

1. Display role select menu (max 25 options)
2. On selection:
   - For each selected role, display modal for label/emoji
   - Add to panel
   - Update edit interface

### Remove Role (`panel:{id}:remove_role`)

1. Display select menu of current panel roles
2. On selection:
   - Remove selected roles from panel
   - Update edit interface
   - If panel is posted, update message

### Style (`panel:{id}:style`)

1. Toggle style: Button <-> SelectMenu
2. Update panel in database
3. Update edit interface
4. If panel is posted, update message

### Color (`panel:{id}:color`)

1. Display select menu with color options:
   - Default (Blurple)
   - Red, Orange, Yellow, Green, Blue, Purple, Pink, Gray
   - Custom (triggers modal for hex input)
2. On selection:
   - Update panel color
   - Update edit interface
   - If panel is posted, update message

### Preview (`panel:{id}:preview`)

1. Build panel embed and components
2. Send as ephemeral message (separate from edit interface)
3. Components should be non-functional in preview

### Post (`panel:{id}:post`)

1. If panel has no roles: show error
2. Display channel select menu
3. On channel selection:
   - Post panel to channel (or update if already posted)
   - Update edit interface to show posted status

### Delete (`panel:{id}:delete`)

1. Display confirmation message with buttons:
   - Confirm (Danger)
   - Cancel (Secondary)
2. On confirm:
   - Delete posted message if exists
   - Delete panel from database
   - Show success message
3. On cancel:
   - Return to edit interface
