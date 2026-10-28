# Protocol (draft)

Transport: WebSocket, localhost, auth token. Encoding: JSON first.
Snapshot rate: 5-10 Hz, delta-based. Include protocol version and sequence numbers.

## Plugin -> Coordinator
- Snapshot { seq, npcs[], nearby_players[] }
- NpcDespawned { id }
- NpcSpawned { id, prefab, loadout, spawn_ts }
- NpcKilled  { id, killer_id, death_ts }   // killer_id absent if not a player

## Coordinator -> Plugin
- MoveTo { id, x, y, z }
- Attack { id, target }
- Hold { id }
- Equip { id, loadout }
- AwardScore { player, points }            // optional, for in-game feedback

Commands must be idempotent.
