# Error Codes

Version: 1.0.0
Last Updated: 2026-01-08

## Application Errors

| Code | Name | Description | User Message |
|------|------|-------------|--------------|
| ERR-001 | NameExists | Panel name already exists in guild | "A panel with this name already exists." |
| ERR-002 | DatabaseError | Database operation failed | "An error occurred. Please try again." |
| ERR-003 | NotFound | Resource not found | "Panel not found." / "Role not found." |
| ERR-004 | DiscordError | Discord API error | "An error occurred. Please try again." |
| ERR-005 | LimitExceeded | Role limit (25) exceeded | "Maximum 25 roles per panel." |
| ERR-006 | Permission | Bot lacks permission | "I don't have permission to manage that role." |
| ERR-007 | InvalidInput | Invalid user input | (specific validation message) |
| ERR-008 | Configuration | Missing or invalid configuration | "Bot is not configured correctly." |

## HTTP Health Check Responses

| Endpoint | Success | Failure |
|----------|---------|---------|
| /health/live | 200 OK | 503 Service Unavailable |
| /health/ready | 200 OK | 503 Service Unavailable |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Normal shutdown |
| 1 | Configuration error |
| 2 | Database connection failed |
| 3 | Discord gateway connection failed |
