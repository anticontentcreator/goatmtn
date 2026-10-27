# Protocol (draft)

Transport: WebSocket, localhost, auth token. Encoding: JSON first.
Snapshot rate: 5-10 Hz, delta-based. Include protocol version and sequence numbers.

## Plugin -> Coordinator
- Snapshot { seq, npcs[], nearby_players[] }
- NpcDespawned { id }

## Coordinator -> Plugin
- MoveTo { id, x, y, z }
- Attack { id, target }
- Hold { id }
- Equip { id, loadout }

Commands must be idempotent.
