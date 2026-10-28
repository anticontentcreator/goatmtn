# Environment reference (goatmtn)

Purpose: one short file of facts Claude should not have to ask for again. No secrets here (no tokens, passwords, keys). Update when something changes; date the line.

Last reviewed: 2026-10-27

## Dev machine
- Windows 10, VS Code, PowerShell 5.1 (note: `>` redirect writes UTF-16; use `| Out-File -Encoding utf8`).
- Repo: `E:\Code\goatmtn` (coordinator/, plugin/, sim/, proto/, docs/).
- Game DLLs (`GameManaged`): `E:\Code\dev\rust_managed\managed`
- Carbon DLLs (`CarbonManaged`): `E:\Code\dev\carbon_managed\managed`
- Machine-specific overrides: `plugin/local.props` (gitignored).
- Decompile output: `E:\Code\dev\decomp` (outside the repo on purpose; never commit decompiled game source).
- ILSpy CLI: `%USERPROFILE%\.dotnet\tools\ilspycmd.cmd` (it is `.cmd`, not `.exe`; may need full path).

## Servers (SSH aliases, key auth from the dev machine)
- `rtest` = test server. All plugin work targets this.
- `rprod` = production. Deploys only via `deploy.ps1 -Target prod` (see gates below).
- Remote user: `rust`. Server root: `/home/rust/server`
- Carbon root: `/home/rust/server/carbon`
  - plugins: `carbon/plugins/` (Carbon loads/reloads files placed here)
  - deploy staging: `carbon/staging/`; backups: `carbon/backups/<timestamp>/`
  - Carbon log: `carbon/logs/Carbon.Core.log`
- Live server log: `/home/rust/server/gm_server.log` (updates in real time; use this for `tail -F`)
- systemd unit: `rust` (`journalctl -u rust`)

## Deploy
- `.\deploy.ps1 -Plugin <Name>` from repo root. Default target is test (rtest).
- Uploads `*.cs` under `plugin/scripts/` (excluding bin/obj), backs up same-named plugins, moves into `carbon/plugins/`.
- Prod gates: clean working tree, on `main` in sync with `origin/main` (or an already-tagged commit), typed `prod` confirmation. Tags `prod-<stamp>` and appends to `backups/DEPLOYS.log`.
- Known stale: the `plugin: deploy` task in `.vscode/tasks.json` uses a hardcoded IP and bare `scp`; prefer `deploy.ps1`.

## Third-party plugins
- Decision: NPCKits for vanilla NPC kits; NpcSpawn only when our plugin must spawn NPCs.
- Not committed to the repo and not placed in `plugin/scripts/` (deploy.ps1 would ship them to prod). Keep in a gitignored folder and install on rtest manually.
- Currently on rtest: GoatHello, NpcSpawn v3.4.8, NPCKits v1.3.0, Kits v4.4.10 (as of 2026-10-27).

## Useful commands
- Watch spawns/kits on rtest:
  `ssh rtest "tail -n 0 -F /home/rust/server/gm_server.log | grep --line-buffered -iE 'GoatHello|kit@'"`
- Check which GoatHello version loaded:
  `ssh rtest "grep -E 'Loaded plugin GoatHello' /home/rust/server/gm_server.log | tail -2"`

## Not recorded here (by design)
- Auth tokens, SSH keys, passwords, RCON credentials. Keep those in `.env` / a password manager.
