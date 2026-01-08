# ADR-001: Discord Library Selection

## Status
Approved

## Context
Need to choose a Rust library for Discord bot development.
Main candidates: Serenity (+Poise) vs Twilight.

## Decision
Use Twilight (twilight-gateway, twilight-http, twilight-model).

## Alternatives Considered

### Serenity + Poise
- Overview: High-level framework with macro-based command definition
- Pros: Easy to use, lots of examples, mature ecosystem
- Cons: Less control, heavier abstraction

### Twilight
- Overview: Modular, low-level Discord library
- Pros: Fine-grained control, excellent K8s support (graceful shutdown), actively maintained
- Cons: More boilerplate, steeper learning curve

## Rationale
- Project targets Kubernetes deployment where graceful shutdown is critical
- Low-level control allows for precise interaction handling
- Active development ensures compatibility with latest Discord API

## Consequences
- More initial boilerplate code
- Better control over gateway lifecycle
- Easier to implement health checks
