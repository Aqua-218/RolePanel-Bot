# /panel list

Version: 1.0.0
Last Updated: 2026-01-08

## Signature

```
/panel list
```

No parameters.

## Permission

User MUST have `Manage Roles` permission.

## Behavior

1. Query all panels for current guild
2. Display list as ephemeral embed

## Response

### Success (with panels)

```
+------------------------------------------+
| Role Panels                              |
+------------------------------------------+
| 1. Role Selection                        |
|    Status: Posted in #general            |
|    Roles: 5                              |
|                                          |
| 2. Color Roles                           |
|    Status: Draft                         |
|    Roles: 8                              |
|                                          |
| 3. Notification Settings                 |
|    Status: Posted in #info               |
|    Roles: 3                              |
+------------------------------------------+
```

### Success (no panels)

```
+------------------------------------------+
| Role Panels                              |
+------------------------------------------+
| No panels found.                         |
| Use /panel create to create one.         |
+------------------------------------------+
```

### Errors

| Condition | Response |
|-----------|----------|
| Missing permission | "You need Manage Roles permission." |
| Internal error | "An error occurred. Please try again." |

## Embed Specification

| Field | Value |
|-------|-------|
| Title | "Role Panels" |
| Color | Blurple (#5865F2) |
| Description | Panel list or "No panels found" message |
| Flags | Ephemeral |
