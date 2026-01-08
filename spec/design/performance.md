# Performance Design

Version: 1.0.0
Last Updated: 2026-01-08

## Performance Goals

| Metric | Target | Maximum |
|--------|--------|---------|
| Interaction acknowledgment | < 500ms | 3000ms |
| Role assignment completion | < 1000ms | 2000ms |
| Panel edit interface display | < 500ms | 1500ms |
| Memory usage (idle) | < 64MB | 128MB |
| Memory usage (active) | < 96MB | 128MB |
| Startup time | < 5s | 10s |

## Hot Paths

### 1. Role Assignment (Button Click)

Most frequent operation. Optimization priority: HIGH.

```
Button Click -> Defer -> DB Query -> Discord API -> Response
              ~50ms     ~10ms        ~200ms        ~50ms
              
Target total: < 500ms
```

Optimizations:
- Defer immediately (acknowledgment within 100ms)
- Single DB query to fetch panel_role
- No caching needed (DB query is fast enough)

### 2. Panel Edit Interface

Moderate frequency. Optimization priority: MEDIUM.

```
Command -> Defer -> DB Query (panel + roles) -> Build Components -> Response
          ~50ms    ~20ms                       ~5ms               ~100ms

Target total: < 500ms
```

Optimizations:
- Use JOIN query to fetch panel with roles in single query
- Pre-build static component templates

### 3. Panel Post

Low frequency. Optimization priority: LOW.

```
Post Button -> Defer -> Build Embed -> Send Message -> Update DB -> Response
              ~50ms    ~5ms           ~300ms          ~10ms        ~50ms

Target total: < 1000ms
```

No specific optimizations needed.

## Database Optimization

### Connection Pool
- Pool size: 5 connections (default)
- Configurable via `DATABASE_MAX_CONNECTIONS`
- Sufficient for single-guild personal use

### Query Optimization

```sql
-- Panel with roles (single query)
SELECT p.*, 
       COALESCE(json_agg(pr.* ORDER BY pr.position) FILTER (WHERE pr.id IS NOT NULL), '[]') as roles
FROM panels p
LEFT JOIN panel_roles pr ON p.id = pr.panel_id
WHERE p.id = $1
GROUP BY p.id;
```

### Indexes
All queries use indexed columns:
- `guild_id` for panel listing
- `message_id` for interaction handling
- `panel_id` for role queries

## Memory Optimization

### Twilight Configuration
- Intents: Minimal (GUILDS only, no message content)
- Cache: Disabled (not needed for this use case)
- No large allocations in hot paths

### String Handling
- Use `&str` where ownership not needed
- Avoid cloning in loops
- Pre-allocate vectors when size is known

## Async Considerations

### Tokio Runtime
- Current-thread runtime (single-threaded)
- Sufficient for personal server use
- Can switch to multi-thread if needed

### Concurrency
- All Discord API calls are async
- Database queries are async
- No blocking operations in async context

## Caching Strategy

### No Caching Required
For personal server use, caching is unnecessary:
- Panel data changes infrequently
- DB queries are fast (< 20ms)
- Caching adds complexity and memory

### Future Consideration
If scaling to public bot:
- Consider caching panel data in memory
- Use LRU cache with 5-minute TTL
- Invalidate on panel update
