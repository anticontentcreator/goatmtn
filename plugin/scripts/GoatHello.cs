using Oxide.Core;

namespace Carbon.Plugins
{
    [Info("GoatHello", "you", "0.0.1")]
    [Description("goatmtn Phase 0: log NPC spawns")]
    public class GoatHello : CarbonPlugin   // VERIFY base class for your Carbon version
    {
        private void OnServerInitialized()
        {
            Puts("GoatHello loaded");
        }

        // Standard Oxide-style hook, fires for every networked entity spawn
        private void OnEntitySpawned(BaseNetworkable entity)   // VERIFY signature
        {
            var npc = entity as ScientistNPC;                  // VERIFY type exists in your build
            if (npc == null) return;

            Puts($"NPC spawned: {npc.ShortPrefabName} id={npc.net.ID} pos={npc.transform.position}");
        }
    }
}