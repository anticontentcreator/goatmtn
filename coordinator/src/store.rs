#![allow(dead_code)]
//! Coordinator-side NPC store. Holds each live NPC's facts from spawn and turns
//! a kill event into an Award via `score::score_kill`.
//!
//! Event shapes are local stand-ins; build them from proto/ types once they exist.
//!
//! Rules under test:
//! - Key is (boot_id, id): the same net.ID in a new boot is a different NPC.
//! - Spawned is idempotent (plugin re-sends it for live NPCs after a reconnect);
//!   the first spawn_ts wins.
//! - A killed NPC is remembered so a stale re-sent Spawned cannot resurrect it
//!   and award twice.
//! - Unknown IDs and duplicate kills are ignored, never a panic.

use std::collections::{HashMap, HashSet};

use crate::score::{score_kill, Award, KillFacts, ScoreConfig};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NpcKey {
    pub boot_id: u64,
    pub id: u64,
}

#[derive(Debug, Clone)]
pub enum Event {
    Spawned {
        key: NpcKey,
        prefab: String,
        loadout: String,
        spawn_ts: f64,
    },
    Killed {
        key: NpcKey,
        killer: Option<u64>,
        death_ts: f64,
    },
    Despawned {
        key: NpcKey,
    },
}

#[derive(Debug, Clone)]
struct NpcFacts {
    prefab: String,
    loadout: String,
    spawn_ts: f64,
}

pub struct NpcStore {
    cfg: ScoreConfig,
    alive: HashMap<NpcKey, NpcFacts>,
    finished: HashSet<NpcKey>,
}

impl NpcStore {
    pub fn new(cfg: ScoreConfig) -> Self {
        Self {
            cfg,
            alive: HashMap::new(),
            finished: HashSet::new(),
        }
    }

    /// Apply one event. Returns an Award only for a player kill of a known, live NPC.
    pub fn apply(&mut self, ev: Event) -> Option<Award> {
        match ev {
            Event::Spawned {
                key,
                prefab,
                loadout,
                spawn_ts,
            } => {
                // A stale re-send of an NPC that already died or despawned must not resurrect it.
                if self.finished.contains(&key) {
                    return None;
                }
                // First spawn wins, so a replay after reconnect cannot reset the clock.
                self.alive.entry(key).or_insert(NpcFacts {
                    prefab,
                    loadout,
                    spawn_ts,
                });
                None
            }
            Event::Killed {
                key,
                killer,
                death_ts,
            } => {
                let facts = self.alive.remove(&key);
                self.finished.insert(key);
                // Unknown ID (e.g. missed the spawn) or duplicate kill: nothing to score.
                let facts = facts?;
                score_kill(
                    &self.cfg,
                    &KillFacts {
                        prefab: facts.prefab,
                        loadout: facts.loadout,
                        spawn_ts: facts.spawn_ts,
                        death_ts,
                        killer,
                    },
                )
            }
            Event::Despawned { key } => {
                self.alive.remove(&key);
                self.finished.insert(key);
                None
            }
        }
    }

    pub fn alive_count(&self) -> usize {
        self.alive.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAYER: u64 = 76561198000000001;

    // Same exact-in-f64 fixture style as score.rs. NOT the production table.
    fn cfg() -> ScoreConfig {
        ScoreConfig {
            base_points: 10.0,
            prefab_mult: HashMap::from([("rhib".to_string(), 1.0)]),
            default_prefab_mult: 1.0,
            loadout_mult: HashMap::from([("Science".to_string(), 1.0)]),
            default_loadout_mult: 1.0,
            points_per_second: 0.5,
            max_seconds: None,
        }
    }

    fn key(boot_id: u64, id: u64) -> NpcKey {
        NpcKey { boot_id, id }
    }

    fn spawned(k: NpcKey, spawn_ts: f64) -> Event {
        Event::Spawned {
            key: k,
            prefab: "rhib".to_string(),
            loadout: "Science".to_string(),
            spawn_ts,
        }
    }

    fn killed(k: NpcKey, killer: Option<u64>, death_ts: f64) -> Event {
        Event::Killed {
            key: k,
            killer,
            death_ts,
        }
    }

    /// Feed events in order, collect every award produced.
    fn replay(store: &mut NpcStore, events: Vec<Event>) -> Vec<Award> {
        events.into_iter().filter_map(|e| store.apply(e)).collect()
    }

    #[test]
    fn player_kill_of_known_npc_awards_points() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                killed(key(1, 5), Some(PLAYER), 1060.0),
            ],
        );
        // 10 + 0.5 * 60 = 40
        assert_eq!(awards, vec![Award { player: PLAYER, points: 40 }]);
    }

    #[test]
    fn kill_without_player_awards_nothing_and_removes_npc() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![spawned(key(1, 5), 1000.0), killed(key(1, 5), None, 1060.0)],
        );
        assert!(awards.is_empty());
        assert_eq!(s.alive_count(), 0);
    }

    #[test]
    fn kill_of_unknown_id_is_ignored() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(&mut s, vec![killed(key(1, 99), Some(PLAYER), 1060.0)]);
        assert!(awards.is_empty());
        assert_eq!(s.alive_count(), 0);
    }

    #[test]
    fn duplicate_kill_awards_once() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                killed(key(1, 5), Some(PLAYER), 1060.0),
                killed(key(1, 5), Some(PLAYER), 1060.0),
            ],
        );
        assert_eq!(awards.len(), 1);
    }

    #[test]
    fn despawn_removes_npc_without_award() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                Event::Despawned { key: key(1, 5) },
                killed(key(1, 5), Some(PLAYER), 1060.0),
            ],
        );
        assert!(awards.is_empty());
        assert_eq!(s.alive_count(), 0);
    }

    #[test]
    fn same_net_id_in_new_boot_is_a_different_npc() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                killed(key(2, 5), Some(PLAYER), 1060.0),
            ],
        );
        assert!(awards.is_empty());
        assert_eq!(s.alive_count(), 1); // boot 1's NPC is untouched
    }

    #[test]
    fn resent_spawn_is_idempotent_and_first_spawn_ts_wins() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                spawned(key(1, 5), 1030.0), // re-send after reconnect, must not reset the clock
                killed(key(1, 5), Some(PLAYER), 1060.0),
            ],
        );
        // 60 s alive from the first spawn_ts: 10 + 0.5 * 60 = 40
        assert_eq!(awards, vec![Award { player: PLAYER, points: 40 }]);
        assert_eq!(s.alive_count(), 0);
    }

    #[test]
    fn resent_spawn_does_not_double_count_alive() {
        let mut s = NpcStore::new(cfg());
        replay(
            &mut s,
            vec![spawned(key(1, 5), 1000.0), spawned(key(1, 5), 1000.0)],
        );
        assert_eq!(s.alive_count(), 1);
    }

    #[test]
    fn stale_resent_spawn_after_kill_does_not_resurrect() {
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                killed(key(1, 5), Some(PLAYER), 1060.0),
                spawned(key(1, 5), 1000.0), // stale replay arriving late
                killed(key(1, 5), Some(PLAYER), 1060.0),
            ],
        );
        assert_eq!(awards.len(), 1);
        assert_eq!(s.alive_count(), 0);
    }

    #[test]
    fn rebuild_after_reconnect_scores_kills_of_replayed_npcs() {
        // Fresh store = coordinator restarted. Plugin replays live NPCs, then a kill arrives.
        let mut s = NpcStore::new(cfg());
        let awards = replay(
            &mut s,
            vec![
                spawned(key(1, 5), 1000.0),
                spawned(key(1, 6), 1010.0),
                killed(key(1, 6), Some(PLAYER), 1070.0),
            ],
        );
        assert_eq!(awards, vec![Award { player: PLAYER, points: 40 }]);
        assert_eq!(s.alive_count(), 1);
    }
}