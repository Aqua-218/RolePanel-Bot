# ADR-002: Database Access Library

## Status
Approved

## Context
Need to choose a Rust library for PostgreSQL database access.
Main candidates: SQLx vs Diesel vs SeaORM.

## Decision
Use SQLx with compile-time query checking.

## Alternatives Considered

### SQLx
- Overview: Async-first SQL toolkit with compile-time checking
- Pros: Native async, lightweight, compile-time SQL verification
- Cons: Raw SQL (no query builder)

### Diesel
- Overview: Full ORM with query DSL
- Pros: Type-safe query builder, mature
- Cons: Async requires separate crate, heavy for simple use case

### SeaORM
- Overview: ActiveRecord-style ORM built on SQLx
- Pros: High-level abstraction, async native
- Cons: Additional abstraction layer, more dependencies

## Rationale
- Twilight uses Tokio, SQLx integrates seamlessly
- Compile-time checking catches SQL errors early
- Simple queries don't need ORM abstraction
- Lightweight for personal server use

## Consequences
- Write raw SQL (acceptable for this project's scope)
- Compile-time query verification requires DB connection during build
- Simple, direct database access
