# goatmtn: NPC coordination project

External coordinator (Rust language) + thin Carbon plugin (C#) controlling NPCs in a Rust (game) server.
Later: headless sim + ML training (Python) with ONNX inference in the coordinator.

## Read first
- docs/spec.md (features, acceptance criteria)
- docs/protocol.md (messages, tick rates)
- docs/decisions.md (settled choices; do not relitigate without asking)
- docs/environment.md (hosts, paths, deploy, logs; no secrets)
- docs/phase0-findings.md (verified vs. assumed; update as work proceeds)

## Current phase
Phase 1: Explore Carbon Loader and plugin capabilities for Custom NPCs and Support for loading items

## Layout
- plugin/scripts/*.cs = deployed plugin sources; plugin/NpcBridge.csproj = build only
- Decompiled game output lives in E:\Code\dev\decomp, never in the repo

## Commands (Windows, run from repo root)
- All checks: task "check all" (Ctrl+Shift+B / Run Task)
- Coordinator: `cd coordinator && cargo test && cargo clippy --all-targets -- -D warnings`
- Plugin: `dotnet build plugin/NpcBridge.csproj`
- Sim: `sim/.venv/Scripts/python.exe -m pytest sim`
- Deploy: `.\deploy.ps1 -Plugin <Name>` (defaults to rtest)

## Rules
- Plugin stays thin: snapshot out, intents in, local execution. No decision logic in the plugin.
- Game objects (BaseEntity/BasePlayer) are touched on the main thread only. Network threads enqueue; main thread drains.
- Never guess Carbon/Rust hook signatures. Ask for the current signature or decompiled type; mark unverified code with `// VERIFY`.
- Protocol types come from proto/ only; do not hand-edit generated types.
- Write the test first. Replay-based tests for coordinator behavior.
- Coordinator must fail safe: on disconnect, NPCs revert to default AI.
- Bind sockets to localhost; require an auth token.

## Safety
- Default target is rtest. Do not run `deploy.ps1 -Target prod` or touch rprod unless asked in that message.
- Third-party plugins (NPCKits, NpcSpawn) never go in plugin/scripts/ (deploy.ps1 would ship them to prod).
- Never commit decompiled game source.
- No secrets in docs, code or chat (tokens, keys, RCON); use .env.
- Diagnostic plugin builds stay on rtest, not main.

## Workflow
- Remove a `// VERIFY` only after the signature compiled or was read from the decompile; then record it in docs/phase0-findings.md.
- Keep phase0-findings.md sections separate (Verified / Inferences / Open questions) and update its date.

## Gotchas
- PowerShell 5.1: `>` writes UTF-16; use `| Out-File -Encoding utf8`.
- ilspycmd is `ilspycmd.cmd`, not `.exe`; use the full path if needed.
- The `plugin: deploy` task in .vscode/tasks.json is stale (hardcoded IP, bare scp); use deploy.ps1.