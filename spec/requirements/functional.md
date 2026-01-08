# Functional Requirements

Version: 1.0.0
Last Updated: 2026-01-08

## FR-001: Panel Creation

Bot MUST provide a slash command to create a new role panel.

### FR-001-1: Create Command
- Command: `/panel create`
- Bot MUST display a modal for entering panel title and description
- Bot MUST create a panel record in database after modal submission
- Bot MUST display an ephemeral edit interface after creation

### FR-001-2: Panel Edit Interface
Bot MUST display an ephemeral message with the following components:
- Current panel information (title, description, registered roles)
- [Add Role] button - Opens role select menu
- [Remove Role] button - Opens registered role select menu for removal
- [Style] button - Toggle between Button/SelectMenu style
- [Color] button - Opens color selection
- [Preview] button - Shows panel preview
- [Post] button - Posts panel to selected channel
- [Delete] button - Deletes panel (with confirmation)

---

## FR-002: Panel Role Management

### FR-002-1: Add Role
- Bot MUST display a role select menu when [Add Role] is clicked
- Bot MUST allow selecting multiple roles at once (max 25)
- Bot MUST display a modal for entering label and emoji for each role
- Bot MUST store role configuration in database
- Bot MUST update the edit interface after adding

### FR-002-2: Remove Role
- Bot MUST display a select menu of registered roles when [Remove Role] is clicked
- Bot MUST allow selecting multiple roles for removal
- Bot MUST remove selected roles from panel configuration
- Bot MUST update the edit interface after removal

### FR-002-3: Role Limit
- Panel MUST support maximum 25 roles (Discord component limit)
- Bot MUST reject addition if limit would be exceeded

---

## FR-003: Panel Style

### FR-003-1: Button Style
- Bot MUST render panel with button components
- Each button MUST display: emoji (optional) + label
- Buttons MUST be arranged in rows (max 5 per row, max 5 rows)
- Clicking a button MUST toggle the associated role

### FR-003-2: Select Menu Style
- Bot MUST render panel with a select menu component
- Select menu MUST allow multiple selections
- Each option MUST display: emoji (optional) + label + description (optional)
- Bot MUST display a [Confirm] button below select menu
- Clicking [Confirm] MUST sync user's roles with selection

### FR-003-3: Style Toggle
- Bot MUST allow switching between Button and SelectMenu style
- Bot MUST preserve role configuration when switching styles

---

## FR-004: Panel Appearance

### FR-004-1: Embed
- Panel MUST be displayed as an embed message
- Embed MUST include: title, description (optional), color
- Embed MUST list all available roles with their labels

### FR-004-2: Color Selection
- Bot MUST provide predefined color options
- Colors: Red, Orange, Yellow, Green, Blue, Purple, Pink, Gray, Default
- Bot MUST allow custom hex color input via modal

---

## FR-005: Panel Posting

### FR-005-1: Channel Selection
- Bot MUST display a channel select menu when [Post] is clicked
- Bot MUST filter to show only text channels where bot has send permission

### FR-005-2: Post Panel
- Bot MUST send the panel embed with components to selected channel
- Bot MUST store the message ID in database for future reference
- Bot MUST update edit interface to show posted status

### FR-005-3: Update Posted Panel
- If panel is already posted, [Post] MUST update the existing message
- Bot MUST NOT create duplicate panel messages

---

## FR-006: Role Assignment

### FR-006-1: Button Click Handler
- Bot MUST toggle role when user clicks a panel button
- If user has role: remove role
- If user does not have role: add role
- Bot MUST respond with ephemeral message indicating result

### FR-006-2: Select Menu Handler
- Bot MUST sync roles when user clicks [Confirm] after selection
- Bot MUST add roles that are selected but user doesn't have
- Bot MUST remove roles that are not selected but user has (panel roles only)
- Bot MUST respond with ephemeral message listing changes

### FR-006-3: Permission Check
- Bot MUST verify it has permission to manage the target role
- Bot MUST verify target role is below bot's highest role
- Bot MUST respond with error message if permission denied

---

## FR-007: Logging

### FR-007-1: User Notification (Ephemeral)
- Bot MUST send ephemeral message to user after role change
- Message MUST include: action (added/removed), role name, timestamp
- Format: "Role added: @RoleName" or "Role removed: @RoleName"

### FR-007-2: Audit Log Channel
- Bot MUST support configuring an audit log channel per guild
- Command: `/config audit-channel <channel>`
- Bot MUST send log message to audit channel after each role change
- Log MUST include: user mention, action, role name, panel name, timestamp
- Log MUST be an embed for readability

### FR-007-3: Audit Log Disable
- Command: `/config audit-channel disable`
- Bot MUST stop sending audit logs when disabled

---

## FR-008: Panel Management

### FR-008-1: List Panels
- Command: `/panel list`
- Bot MUST display all panels in the guild
- Display MUST include: panel name, status (posted/draft), role count

### FR-008-2: Edit Existing Panel
- Command: `/panel edit <name>`
- Bot MUST display the edit interface for specified panel
- Bot MUST support autocomplete for panel names

### FR-008-3: Delete Panel
- [Delete] button in edit interface MUST show confirmation
- Bot MUST delete panel message if posted
- Bot MUST delete panel record and associated data from database

---

## FR-009: Configuration

### FR-009-1: Audit Channel Configuration
- See FR-007-2 and FR-007-3

### FR-009-2: View Configuration
- Command: `/config show`
- Bot MUST display current guild configuration
- Display MUST include: audit channel (if set)

---

## FR-010: Error Handling

### FR-010-1: Permission Errors
- Bot MUST display clear error message when lacking permissions
- Error MUST be ephemeral
- Error MUST specify which permission is missing

### FR-010-2: Not Found Errors
- Bot MUST display error when panel/role not found
- Error MUST be ephemeral

### FR-010-3: Rate Limit Handling
- Bot MUST handle Discord rate limits gracefully
- Bot MUST NOT crash on rate limit errors
- Bot MUST retry with appropriate backoff
