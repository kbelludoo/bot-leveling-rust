use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(rename = "localIp", default)]
    pub local_ip: String,
    #[serde(default)]
    pub proxy: String,
    pub credentials: GameCredentials,
    #[serde(default)]
    pub settings: GameSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameCredentials {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub token: String,
    #[serde(rename = "deviceUuid", default)]
    pub device_uuid: String,
    #[serde(default)]
    pub origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    #[serde(rename = "huntTarget", default = "default_hunt_target")]
    pub hunt_target: String,
    #[serde(rename = "autoHunt", default = "default_true")]
    pub auto_hunt: bool,
    #[serde(rename = "autoSell", default = "default_true")]
    pub auto_sell: bool,
    #[serde(rename = "autoHeal", default = "default_true")]
    pub auto_heal: bool,
    #[serde(rename = "autoDaily", default = "default_true")]
    pub auto_daily: bool,
    #[serde(rename = "autoEquip", default = "default_true")]
    pub auto_equip: bool,
    #[serde(rename = "autoRepair", default = "default_true")]
    pub auto_repair: bool,
    #[serde(rename = "autoShadows", default = "default_true")]
    pub auto_shadows: bool,
    #[serde(rename = "autoTower", default = "default_true")]
    pub auto_tower: bool,
    #[serde(rename = "autoDailyBoss", default = "default_true")]
    pub auto_daily_boss: bool,
    #[serde(rename = "autoGems", default = "default_true")]
    pub auto_gems: bool,
    #[serde(rename = "autoEnchant", default = "default_true")]
    pub auto_enchant: bool,
    #[serde(rename = "autoStats", default = "default_true")]
    pub auto_stats: bool,
    #[serde(rename = "autoMissions", default = "default_true")]
    pub auto_missions: bool,
    #[serde(rename = "autoPromotion", default = "default_true")]
    pub auto_promotion: bool,
    #[serde(rename = "partyLeader", default)]
    pub party_leader: bool,
    #[serde(rename = "partyMembers", default)]
    pub party_members: Vec<String>,
    #[serde(rename = "autoParty", default = "default_true")]
    pub auto_party: bool,
    #[serde(rename = "codexMode", default = "default_true")]
    pub codex_mode: bool,
    #[serde(rename = "codexTarget", default = "default_codex_target")]
    pub codex_target: u64,
    #[serde(rename = "goldReserve", default = "default_gold_reserve")]
    pub gold_reserve: i64,
    #[serde(rename = "preferredClass", default)]
    pub preferred_class: String,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            hunt_target: default_hunt_target(),
            auto_hunt: true,
            auto_sell: true,
            auto_heal: true,
            auto_daily: true,
            auto_equip: true,
            auto_repair: true,
            auto_shadows: true,
            auto_tower: true,
            auto_daily_boss: true,
            auto_gems: true,
            auto_enchant: true,
            auto_stats: true,
            auto_missions: true,
            auto_promotion: true,
            party_leader: false,
            party_members: Vec::new(),
            auto_party: true,
            codex_mode: true,
            codex_target: 2000,
            gold_reserve: 100000,
            preferred_class: String::new(),
        }
    }
}

fn default_hunt_target() -> String {
    "floresta-sombria".to_string()
}
fn default_true() -> bool {
    true
}
fn default_codex_target() -> u64 {
    2000
}
fn default_gold_reserve() -> i64 {
    100000
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStatus {
    pub name: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub rank: String,
    #[serde(default)]
    pub vocation: String,
    #[serde(default)]
    pub hp: u32,
    #[serde(default)]
    pub max_hp: u32,
    #[serde(default)]
    pub mana: u32,
    #[serde(default)]
    pub max_mana: u32,
    #[serde(default)]
    pub gold: i64,
    #[serde(default)]
    pub current_area: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub kills: u64,
    #[serde(default)]
    pub exp_per_hour: f64,
    #[serde(default)]
    pub duration_sec: u64,
    #[serde(default)]
    pub codex_info: String,
    #[serde(default)]
    pub logs: Vec<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}
