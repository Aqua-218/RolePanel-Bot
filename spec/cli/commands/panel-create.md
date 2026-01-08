# /panel create

Version: 1.0.0
Last Updated: 2026-01-08

## Signature

```
/panel create
```

No parameters. Opens modal for input.

## Permission

User MUST have `Manage Roles` permission.

## Behavior

1. Bot displays modal with fields:
   - Title (required, max 100 chars)
   - Description (optional, max 4096 chars)

2. On modal submit:
   - Validate inputs
   - Create panel in database
   - Display edit interface (ephemeral)

## Modal Specification

### Modal ID
`panel:create:modal`

### Fields

| Field | Label | Style | Required | Max Length | Placeholder |
|-------|-------|-------|----------|------------|-------------|
| title | Title | Short | Yes | 100 | Panel title |
| description | Description | Paragraph | No | 4096 | Panel description (optional) |

## Response

### Success
Ephemeral message with edit interface. See [panel-edit-interface.md](../design/components/panel-edit/interface.md).

### Errors

| Condition | Response |
|-----------|----------|
| Name already exists | "A panel with this name already exists." |
| Missing permission | "You need Manage Roles permission." |
| Internal error | "An error occurred. Please try again." |

## Example Flow

```
User: /panel create
Bot: [Modal: Create Panel]
     - Title: [________________]
     - Description: [________________]
     
User: [Submits modal]
     - Title: "Role Selection"
     - Description: "Choose your roles!"
     
Bot: [Ephemeral Message]
     Panel: Role Selection
     Description: Choose your roles!
     Style: Button
     Roles: None
     
     [Add Role] [Remove Role] [Style]
     [Color] [Preview] [Post]
     [Delete]
```
