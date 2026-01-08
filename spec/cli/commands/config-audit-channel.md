# /config audit-channel

Version: 1.0.0
Last Updated: 2026-01-08

## Signature

```
/config audit-channel [channel]
```

## Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| channel | Channel | No | Audit log channel. Omit to disable. |

## Permission

User MUST have `Administrator` permission.

## Behavior

### Set Channel
If `channel` is provided:
1. Validate channel is text channel
2. Validate bot has Send Messages permission in channel
3. Save channel ID to guild config
4. Confirm with ephemeral message

### Disable
If `channel` is omitted:
1. Remove audit channel from guild config
2. Confirm with ephemeral message

## Response

### Success (set)
"Audit log channel set to #channel-name."

### Success (disable)
"Audit logging disabled."

### Errors

| Condition | Response |
|-----------|----------|
| Not a text channel | "Please select a text channel." |
| Missing permission in channel | "I don't have permission to send messages in that channel." |
| Missing permission (user) | "You need Administrator permission." |
| Internal error | "An error occurred. Please try again." |
