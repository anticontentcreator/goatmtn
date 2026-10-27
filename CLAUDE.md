# goatmtn: NPC coordination project

External coordinator (Rust language) + thin Carbon plugin (C#) controlling NPCs in a Rust (game) server.
Later: headless sim + ML training (Python) with ONNX inference in the coordinator.

## Read first
- docs/spec.md (features, acceptance criteria)
- docs/protocol.md (messages, tick rates)
- docs/decisions.md (settled choices; do not relitigate without asking)

## Commands (Windows, run from repo root)
- All checks: task "check all" (Ctrl+Shift+B / Run Task)
- Coordinator: `cd coordinator && cargo test && cargo clippy --all-targets -- -D warnings`
- Plugin: `dotnet build plugin/NpcBridge.csproj`
- Sim: `sim/.venv/Scripts/python.exe -m pytest sim`

## Rules
- Plugin stays thin: snapshot out, intents in, local execution. No decision logic in the plugin.
- Game objects (BaseEntity/BasePlayer) are touched on the main thread only. Network threads enqueue; main thread drains.
- Never guess Carbon/Rust hook signatures. Ask for the current signature or decompiled type; mark unverified code with `// VERIFY`.
- Protocol types come from proto/ only; do not hand-edit generated types.
- Write the test first. Replay-based tests for coordinator behavior.
- Coordinator must fail safe: on disconnect, NPCs revert to default AI.
- Bind sockets to localhost; require an auth token.
