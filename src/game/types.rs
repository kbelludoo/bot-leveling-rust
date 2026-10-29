use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameSettings {
    #[serde(rename = "huntTarget", default)]
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
    #[serde(rename = "preferredClass", default)]
    pub preferred_class: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CharacterStatus {
    pub name: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub rank: String,
    #[serde(default)]
    pub hp: u32,
    #[serde(default)]
    pub max_hp: u32,
    #[serde(default)]
    pub gold: i64,
    #[serde(default)]
    pub shadows_count: u32,
}
