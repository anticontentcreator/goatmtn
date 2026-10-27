# Spec

## Goals
1. Load named gearsets onto NPCs at spawn.
2. Coordinate groups of NPCs from an external Rust service.
3. (Later) ML-driven strategies trained in a headless sim.

## Features
### F1: Gearset loader
- Acceptance: a JSON kit applies to an NPC's wear/belt containers on spawn.
- Open questions: which NPC types? kit selection rules?

### F2: Bridge
- Acceptance: plugin streams snapshots; coordinator intents move an NPC; disconnect restores default AI.

## Non-goals (for now)
- Live learning on the production server.
