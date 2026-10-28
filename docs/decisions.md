# Decisions
- Framework: Carbon, written against the Oxide-compatible API.
- Coordinator: Rust (language), out-of-process. No FFI initially.
- High-level discrete intents; plugin executes movement/aiming locally.
- Utility-AI baseline before any ML.

## NPC scoring
- Identity: per-spawn only, (boot_id, net.ID) mapped to a coordinator GUID. No persistence across restarts.
- Score computed in the coordinator; plugin reports facts only.
- Kill attribution: last hit by a player. NPC, environmental and self kills award nothing.
- Time-alive term is linear, config-driven (k). No cap for now.
- Inputs: loadout (snapshot at spawn), prefab-based difficulty table, time alive.

## Plugin references
Game: Assembly-CSharp, UnityEngine.CoreModule, Facepunch.UnityEngine, Facepunch.Network, Rust.Data
Carbon: Carbon.Common, Carbon, Carbon.SDK
Paths set in plugin/NpcBridge.csproj (GameManaged, CarbonManaged)