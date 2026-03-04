# trying-message-passing-actors-rs

Archival snapshot of a Rust/Tokio experiment for message-passing actor-style simulation.

## What this project was trying to do

Build a high-churn async simulation using channels instead of shared mutable state:

- One global `broadcast` price feed emits random-walk ticks for crypto symbols.
- Multiple users each own many device actors.
- Each device subscribes to prices, watches one symbol, and emits threshold-crossing events.
- Parent/child relationships between devices are modeled through user-scoped event buses.
- User sessions flap online/offline while devices continue running, and device population churns over time.

Core files:

- `src/main.rs`: bootstraps feed + users and runs the simulation window.
- `src/price.rs`: synthetic market data producer.
- `src/user.rs`: per-user controller, device lifecycle/churn, session behavior.
- `src/device.rs`: actor loop handling price updates, commands, and child-event observation.
- `src/types.rs`: shared message/event/config types.
- `spec.md`: design notes and goals.

## When work happened

Source project (`mpsc-test`) had no commits, so timing here is reconstructed from file modification timestamps.

- **Primary coding session:** September 21, 2025 (local machine time, US/Central)
- **Earliest file timestamp:** 2025-09-21 12:01 (`.gitignore`)
- **Latest source edit timestamp:** 2025-09-21 12:55 (`src/user.rs`)
- **Archive repo created / snapshot copied:** 2026-03-04

This repo is intended as an archive/reference snapshot of that experiment.
