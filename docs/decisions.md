# Decisions
- Framework: Carbon, written against the Oxide-compatible API.
- Coordinator: Rust (language), out-of-process. No FFI initially.
- High-level discrete intents; plugin executes movement/aiming locally.
- Utility-AI baseline before any ML.

## Plugin references
Game: Assembly-CSharp, UnityEngine.CoreModule, Facepunch.UnityEngine, Facepunch.Network, Rust.Data
Carbon: Carbon.Common, Carbon, Carbon.SDK
Paths set in plugin/NpcBridge.csproj (GameManaged, CarbonManaged)