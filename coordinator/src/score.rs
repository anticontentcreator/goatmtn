#![allow(dead_code)]
//! NPC score function. Pure: no I/O, no game types, no clock.
//!
//! score = base_points * prefab_mult * loadout_mult + points_per_second * seconds_alive
//! Awarded only when a player landed the killing blow (last hit).
//!
//! NOTE: `KillFacts` is a local stand-in. Once `NpcKilled` / `NpcSpawned` exist in
//! proto/, build `KillFacts` from the generated types; do not hand-edit those.
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub base_points: f64,
    pub prefab_mult: HashMap<String, f64>,
    pub default_prefab_mult: f64,
    pub loadout_mult: HashMap<String, f64>,
    pub default_loadout_mult: f64,
    pub points_per_second: f64,
    /// None = uncapped (current decision). Present so a cap needs no code change.
    pub max_seconds: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct KillFacts {
    pub prefab: String,
    /// Snapshot taken at spawn, not at death.
    pub loadout: String,
    /// Plugin clock, seconds.
    pub spawn_ts: f64,
    pub death_ts: f64,
    /// None for NPC, environmental or otherwise non-player kills.
    pub killer: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Award {
    pub player: u64,
    pub points: u64,
}

pub fn score_kill(cfg: &ScoreConfig, kill: &KillFacts) -> Option<Award> {
    // Last hit by a player only; NPC, environmental and self kills award nothing.
    let player = kill.killer?;

    let prefab = cfg
        .prefab_mult
        .get(&kill.prefab)
        .copied()
        .unwrap_or(cfg.default_prefab_mult);
    let loadout = cfg
        .loadout_mult
        .get(&kill.loadout)
        .copied()
        .unwrap_or(cfg.default_loadout_mult);

    // Clock skew or bad data counts as zero seconds, never negative.
    let mut secs = (kill.death_ts - kill.spawn_ts).max(0.0);
    if let Some(cap) = cfg.max_seconds {
        secs = secs.min(cap);
    }

    let raw = cfg.base_points * prefab * loadout + cfg.points_per_second * secs;
    let points = raw.max(0.0).round() as u64;

    Some(Award { player, points })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixture values chosen to be exact in f64. They are NOT the production table.
    fn cfg() -> ScoreConfig {
        ScoreConfig {
            base_points: 10.0,
            prefab_mult: HashMap::from([
                ("rhib".to_string(), 1.0),
                ("cargo".to_string(), 2.0),
            ]),
            default_prefab_mult: 1.0,
            loadout_mult: HashMap::from([
                ("Science".to_string(), 1.0),
                ("Science Pistol".to_string(), 0.5),
            ]),
            default_loadout_mult: 1.0,
            points_per_second: 0.5,
            max_seconds: None,
        }
    }

    fn kill(prefab: &str, loadout: &str, alive_secs: f64) -> KillFacts {
        KillFacts {
            prefab: prefab.to_string(),
            loadout: loadout.to_string(),
            spawn_ts: 1000.0,
            death_ts: 1000.0 + alive_secs,
            killer: Some(76561198000000001),
        }
    }

    fn points(cfg: &ScoreConfig, k: &KillFacts) -> u64 {
        score_kill(cfg, k).expect("expected an award").points
    }

    #[test]
    fn no_player_killer_awards_nothing() {
        let mut k = kill("rhib", "Science", 60.0);
        k.killer = None;
        assert_eq!(score_kill(&cfg(), &k), None);
    }

    #[test]
    fn award_goes_to_the_killer() {
        let k = kill("rhib", "Science", 0.0);
        let award = score_kill(&cfg(), &k).unwrap();
        assert_eq!(award.player, 76561198000000001);
    }

    #[test]
    fn instant_kill_scores_base_only() {
        assert_eq!(points(&cfg(), &kill("rhib", "Science", 0.0)), 10);
    }

    #[test]
    fn sixty_seconds_alive_adds_thirty() {
        // 10 * 1.0 * 1.0 + 0.5 * 60 = 40
        assert_eq!(points(&cfg(), &kill("rhib", "Science", 60.0)), 40);
    }

    #[test]
    fn time_term_is_linear() {
        let c = cfg();
        let p0 = points(&c, &kill("rhib", "Science", 0.0));
        let p60 = points(&c, &kill("rhib", "Science", 60.0));
        let p120 = points(&c, &kill("rhib", "Science", 120.0));
        assert_eq!(p60 - p0, p120 - p60);
    }

    #[test]
    fn prefab_multiplier_scales_base_not_time() {
        // 10 * 2.0 * 1.0 + 0.5 * 60 = 50
        assert_eq!(points(&cfg(), &kill("cargo", "Science", 60.0)), 50);
    }

    #[test]
    fn loadout_multiplier_scales_base_not_time() {
        // 10 * 1.0 * 0.5 + 0.5 * 60 = 35
        assert_eq!(points(&cfg(), &kill("rhib", "Science Pistol", 60.0)), 35);
    }

    #[test]
    fn unknown_prefab_and_loadout_use_defaults() {
        assert_eq!(
            points(&cfg(), &kill("mystery", "Nothing", 0.0)),
            10
        );
    }

    #[test]
    fn death_before_spawn_counts_as_zero_seconds() {
        let mut k = kill("rhib", "Science", 0.0);
        k.death_ts = k.spawn_ts - 5.0; // clock skew or bad data
        assert_eq!(points(&cfg(), &k), 10);
    }

    #[test]
    fn optional_cap_limits_time_term() {
        let mut c = cfg();
        c.max_seconds = Some(100.0);
        // 10 + 0.5 * 100 = 60, regardless of 1000 s alive
        assert_eq!(points(&c, &kill("rhib", "Science", 1000.0)), 60);
    }

    #[test]
    fn uncapped_by_default_keeps_growing() {
        // 10 + 0.5 * 1000 = 510
        assert_eq!(points(&cfg(), &kill("rhib", "Science", 1000.0)), 510);
    }

    #[test]
    fn fractional_points_round_to_nearest() {
        let mut c = cfg();
        c.points_per_second = 0.3;
        // 10 + 0.3 * 5 = 11.5 -> 12 (round half away from zero)
        assert_eq!(points(&c, &kill("rhib", "Science", 5.0)), 12);
    }
}