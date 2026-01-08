# /config show

Version: 1.0.0
Last Updated: 2026-01-08

## Signature

```
/config show
```

No parameters.

## Permission

User MUST have `Administrator` permission.

## Behavior

1. Get guild configuration
2. Display as ephemeral embed

## Response

### Success

```
+------------------------------------------+
| Server Configuration                     |
+------------------------------------------+
| Audit Log Channel: #audit-logs           |
| (or "Not configured")                    |
+------------------------------------------+
```

### Errors

| Condition | Response |
|-----------|----------|
| Missing permission | "You need Administrator permission." |
| Internal error | "An error occurred. Please try again." |

## Embed Specification

| Field | Value |
|-------|-------|
| Title | "Server Configuration" |
| Color | Blurple (#5865F2) |
| Fields | Audit Log Channel |
| Flags | Ephemeral |
