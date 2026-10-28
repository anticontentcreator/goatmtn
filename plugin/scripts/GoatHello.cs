using System.Collections.Generic;
using System.Text;
using Oxide.Core;

namespace Carbon.Plugins
{
    [Info("GoatHello", "you", "0.0.6")]
    [Description("goatmtn Phase 0/1: log NPC spawns, loadout picks, kit changes over time, and NPC deaths with killer")]
    public class GoatHello : CarbonPlugin
    {
        private class SpawnInfo
        {
            public string Prefab;
            public string Loadout;
            public float SpawnTime; // seconds, plugin clock
        }

        // Keyed by net.ID.ToString() so this does not depend on the shape of net.ID's type.
        // Main thread only. Entries are removed on death; despawns without death are not
        // cleaned up yet (needs a despawn hook, signature not verified) so this leaks slowly.
        private readonly Dictionary<string, SpawnInfo> spawned = new Dictionary<string, SpawnInfo>();

        private void OnServerInitialized()
        {
            Puts("GoatHello loaded");
        }

        private void OnEntitySpawned(BaseNetworkable entity)
        {
            var npc = entity as ScientistNPC;
            if (npc == null) return;

            var id = npc.net.ID;
            Puts($"NPC spawned: {npc.ShortPrefabName} id={id} pos={npc.transform.position}");
            Puts($"[kit@spawn] {Describe(npc)}");

            // v0.0.6: remember facts at spawn so a later kill can be scored.
            spawned[id.ToString()] = new SpawnInfo
            {
                Prefab = npc.ShortPrefabName,
                Loadout = npc.GetLoadoutName(),
                SpawnTime = UnityEngine.Time.realtimeSinceStartup // VERIFY: compiles in the plugin build
            };

            timer.Once(0.03f, () => Snapshot(npc, id, "0.03s"));
            timer.Once(0.06f, () => Snapshot(npc, id, "0.06s"));
            timer.Once(0.1f,  () => Snapshot(npc, id, "0.1s"));
            timer.Once(0.3f,  () => Snapshot(npc, id, "0.3s"));
            timer.Once(0.6f,  () => Snapshot(npc, id, "0.6s"));
            timer.Once(1f,    () => Snapshot(npc, id, "1s"));
            timer.Once(5f,    () => Snapshot(npc, id, "5s"));
        }

        // VERIFY: Oxide-style signature (OnEntityDeath(BaseCombatEntity, HitInfo)); Carbon dispatch
        // not yet confirmed. Remove this marker only after it compiles AND fires on rtest.
        private void OnEntityDeath(BaseCombatEntity entity, HitInfo info)
        {
            var npc = entity as ScientistNPC;
            if (npc == null) return;

            var key = npc.net.ID.ToString();
            SpawnInfo si;
            bool known = spawned.TryGetValue(key, out si);
            spawned.Remove(key);
            float now = UnityEngine.Time.realtimeSinceStartup;

            // HitInfo.Initiator (BaseEntity field) and InitiatorPlayer (BasePlayer property) read from the decompile.
            BaseEntity initiator = info != null ? info.Initiator : null;
            BasePlayer killer = info != null ? info.InitiatorPlayer : null;
            bool killerIsNpc = killer is NPCPlayer;

            string aliveStr = known ? (now - si.SpawnTime).ToString("F1") : "?";
            string spawnLoadout = known ? si.Loadout : "?";
            string initiatorStr = initiator != null ? initiator.ShortPrefabName : "none";
            string killerId = killer != null ? killer.userID.ToString() : "none"; // VERIFY: BasePlayer.userID

            Puts($"[kill] id={key} prefab={npc.ShortPrefabName} spawnLoadout={spawnLoadout} alive={aliveStr}s infoNull={info == null} initiator={initiatorStr} killerPlayer={killer != null} killerIsNpc={killerIsNpc} killerId={killerId}");
            Puts($"[kit@death] {Describe(npc)}");
        }

        private void Snapshot(ScientistNPC npc, object id, string tag)
        {
            if (npc == null || npc.IsDestroyed)
            {
                Puts($"[kit@{tag}] id={id} gone");
                return;
            }
            Puts($"[kit@{tag}] {Describe(npc)}");
        }

        // Main thread only: reads game objects.
        private static string Describe(ScientistNPC npc)
        {
            var inv = npc.inventory;
            if (inv == null) return $"{npc.ShortPrefabName} id={npc.net.ID} inventory=null";

            var sb = new StringBuilder();
            sb.Append($"{npc.ShortPrefabName} id={npc.net.ID} loadout={npc.GetLoadoutName()} dead={npc.IsDead()} wear=[");
            Append(sb, inv.containerWear);
            sb.Append("] belt=[");
            Append(sb, inv.containerBelt);
            sb.Append("] main=[");
            Append(sb, inv.containerMain);
            sb.Append("]");
            return sb.ToString();
        }

        private static void Append(StringBuilder sb, ItemContainer container)
        {
            if (container == null) { sb.Append("null"); return; }
            bool first = true;
            foreach (Item item in container.itemList)
            {
                if (!first) sb.Append(", ");
                first = false;
                sb.Append(item.info.shortname);
                sb.Append('x');
                sb.Append(item.amount);
            }
        }
    }
}