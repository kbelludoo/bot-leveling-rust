use crate::game::catalog::CATALOG_TIERS;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct PartyCoordinator {
    pub leader_gate: String,
    pub leader_tier: u32,
    pub leader_dif: String,
    pub levels: HashMap<String, u32>,
    pub golds: HashMap<String, i64>,
    pub monster_kills: HashMap<String, HashMap<String, u64>>,
    pub locked_hunts: HashMap<String, Instant>,
}

pub static GLOBAL_PARTY: Lazy<RwLock<PartyCoordinator>> = Lazy::new(|| RwLock::new(PartyCoordinator::default()));

impl PartyCoordinator {
    pub fn mark_hunt_locked(&mut self, key: &str, duration: Duration) {
        self.locked_hunts.insert(key.to_string(), Instant::now() + duration);
    }

    pub fn get_locked_hunts(&self) -> HashMap<String, Instant> {
        let mut cp = HashMap::new();
        let now = Instant::now();
        for (k, v) in &self.locked_hunts {
            if *v > now {
                cp.insert(k.clone(), *v);
            }
        }
        cp
    }

    pub fn report_level(&mut self, bot_id: &str, level: u32) {
        if level > 0 {
            self.levels.insert(bot_id.to_string(), level);
        }
    }

    pub fn report_gold(&mut self, bot_id: &str, gold: i64) {
        if gold >= 0 {
            self.golds.insert(bot_id.to_string(), gold);
        }
    }

    pub fn report_kills(&mut self, bot_id: &str, kills: &HashMap<String, u64>) {
        let entry = self.monster_kills.entry(bot_id.to_string()).or_default();
        for (k, v) in kills {
            entry.insert(k.clone(), *v);
        }
    }

    pub fn get_min_level(&self, fallback: u32) -> u32 {
        if self.levels.is_empty() {
            return fallback;
        }
        let mut min_lvl = 999;
        for &lvl in self.levels.values() {
            if lvl > 0 && lvl < min_lvl {
                min_lvl = lvl;
            }
        }
        if min_lvl == 999 { fallback } else { min_lvl }
    }

    pub fn get_shared_codex_kills(&self) -> HashMap<String, u64> {
        let mut min_kills = HashMap::new();
        for tier in CATALOG_TIERS {
            let mut lowest: u64 = u64::MAX;
            let mut found_any = false;
            for bot_map in self.monster_kills.values() {
                found_any = true;
                let k = bot_map.get(tier.monster_id).copied().unwrap_or(0);
                if k < lowest {
                    lowest = k;
                }
            }
            if !found_any || lowest == u64::MAX {
                lowest = 0;
            }
            min_kills.insert(tier.monster_id.to_string(), lowest);
        }
        min_kills
    }

    pub fn set_leader_hunt(&mut self, gate: &str, tier: u32, dif: &str) {
        self.leader_gate = gate.to_string();
        self.leader_tier = tier;
        self.leader_dif = dif.to_string();
    }

    pub fn get_leader_hunt(&self) -> (String, u32, String) {
        (self.leader_gate.clone(), self.leader_tier, self.leader_dif.clone())
    }

    pub fn get_pool_stats(&self, reserve_per_bot: i64) -> (i64, i64, i64) {
        let mut total: i64 = 0;
        let mut reserved: i64 = 0;
        let mut investable: i64 = 0;
        for &g in self.golds.values() {
            total += g;
            if g > reserve_per_bot {
                reserved += reserve_per_bot;
                investable += g - reserve_per_bot;
            } else {
                reserved += g;
            }
        }
        (total, reserved, investable)
    }
}
