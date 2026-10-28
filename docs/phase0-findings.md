# Phase 0 findings (goatmtn)

Last updated: 2026-10-27. Source of truth for what is verified vs. assumed. Update as work proceeds.

## Verified by compile + observation (GoatHello v0.0.3 on rtest)
- `npc.IsDead()`: compiles; returned False on all logged NPCs (True case not yet observed).
- `npc.GetLoadoutName()`: compiles; returned "Science" (rhib) and "Science Pistol" (junkpile). Also read from the decompile.

## Environment
- `rtest` = test server, `rprod` = production (SSH aliases, key auth). Plugin work targets rtest only.
- Deploy: `.\deploy.ps1 -Plugin GoatHello` (defaults to test). Prod requires clean tree, `main` in sync with `origin/main` (or a tagged commit), and typed confirmation.
- Plugin sources live in `plugin/scripts/`.
- Server log to watch: `/home/rust/server/gm_server.log` (real time). Carbon also writes `carbon/logs/Carbon.Core.log`.
- Decompiling: `ilspycmd` shim is `%USERPROFILE%\.dotnet\tools\ilspycmd.cmd` (not `.exe`). Output goes to `E:\Code\dev\decomp`, outside the repo (do not commit decompiled game source).

## Verified by compile + observation (GoatHello v0.0.2 on rtest)
- `CarbonPlugin` base class, `OnEntitySpawned(BaseNetworkable)` hook, `ScientistNPC` cast.
- `npc.net.ID`, `npc.transform.position`, `npc.ShortPrefabName`.
- `timer.Once(float, Action)`, `IsDestroyed`, `item.info.shortname`, `item.amount`.
- `npc.inventory.containerWear` / `containerBelt`, enumerated via `itemList`.

## Verified from decompile (Assembly-CSharp)
- `BasePlayer.inventory` is a `PlayerInventory`; containers `containerMain`, `containerWear`, `containerBelt`.
- `NPCPlayer`: `loadouts` (`PlayerInventoryProperties[]`), `EquipLoadout(...)`, `GetLoadoutName()`.
- `NPCPlayer.ServerInit` calls `EquipLoadout(loadouts)`, then `Invoke(EquipTest, 0.25f)`.
- `EquipLoadout` picks one entry at random and calls `GiveToPlayer(this)` on it.
- The game has its own `Inventory.LoadLoadout(...)`, `inventory.GiveDefaultItems()` and `inventory.Strip()` in `BasePlayer` (usage seen around lines 12340, 12525, 12533 of `BasePlayer.cs`; not yet read in detail).
- BaseCombatEntity.OnDied(HitInfo info) at line 883 and Die(HitInfo info = null) at line 902, both public virtual. info may be null.
- HitInfo.Initiator is a public BaseEntity field. HitInfo.InitiatorPlayer is a BasePlayer property returning Initiator.ToPlayer() (null guard at line 84).

## Observations (rtest, ScientistNPC only)
- 301 spawn lines logged across several plugin reloads (cumulative, not a rate).
- Variants seen: rhib 112, roam_nvg_variant 46, junkpile_pistol 43, ptboat 37, cargo 20, roam 14, patrol 12, roamtethered 7, cargo_turret_lr300 6, cargo_turret_any 4.
- Not seen: oil rig, heavy, tunnel/underwater dwellers. May be unspawned, disabled, or different classes that the `ScientistNPC` cast does not match.
- Default kit is already present when `OnEntitySpawned` fires (rhib: `hazmatsuit_scientist` in wear, `rifle.lr300` or `smg.mp5` in belt). The lr300/mp5 split is consistent with the random loadout pick.
- 3 of 13 sampled rhib NPCs (ids 737265, 737297, 737329) had an empty belt at +1s that was populated at spawn. Cause unknown. Sample is small and rhib-only.
- 2026-10-27: rhib groups (e.g. ids 753206/753238/753262): belt populated at spawn (smg.mp5 / rifle.lr300), empty at +1s and +5s; main empty, wear intact, dead=False. Group-correlated (consecutive ids, ~15-30 m apart). Cause unknown.
- junkpile_pistol (3 samples): kit unchanged at spawn/1s/5s.

## Inferences (not confirmed)
- `EquipTest` (0.25 s after init) likely reads belt slot 0 and activates it (`UpdateActiveItem`). Body not yet read.
- If so, a gearset applied synchronously in `OnEntitySpawned` would be activated by `EquipTest`; applied later, we would need to activate the weapon ourselves.
- Carbon dispatches OnEntityDeath(BaseCombatEntity, HitInfo) as in Oxide. Source: Oxide community thread, not Carbon docs.
- NPCPlayer derives from BasePlayer, so InitiatorPlayer can be an NPC. Needs an NPCPlayer check before awarding.

## Open questions
- Why do some belts empty within 1 s (moved to main? NPC dead? another plugin? game logic)? v0.0.3 logs `loadout=`, `dead=`, `main=` and a 5 s snapshot to answer this.
- Can a `PlayerInventoryProperties` be built at runtime, or only loaded as a prefab asset? Decides whether F1 uses the game's loadout path or creates items directly.
- Which NPC classes beyond `ScientistNPC` should F1 cover?
- Kit selection key: `ShortPrefabName` (exact or prefix, e.g. `scientistnpc_cargo*`) is the current candidate.
- How to enumerate live NPCs (needed for the reconnect replay).
- Lines 880-960 of BaseCombatEntity.cs (what Die does around OnDied) not yet read.

## Next steps
1. Deploy v0.0.3 to rtest, collect `kit@` lines, explain the empty belts.
2. Read `EquipTest` (NPCPlayer.cs ~line 319-350) and `PlayerInventoryProperties.GiveToPlayer`.
3. Decide F1 approach, then write the gearset loader test-first.

## Cautions
- `// VERIFY` convention: only remove a marker once the signature has compiled or been read from the decompile.
- The v0.0.3 diagnostic logs 3 lines per NPC; keep it on rtest and off `main`.
