use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct HuntTier {
    pub gate_id: &'static str,
    pub gate_name: &'static str,
    pub tier_min: u32,
    pub tier_max: u32,
    pub monster_id: &'static str,
    pub monster_name: &'static str,
    pub boss_name: &'static str,
    pub difficulty: &'static str,
    pub area_level: u32,
    pub required_level: u32,
}

pub static CATALOG_TIERS: &[HuntTier] = &[
    // --- Normal (Níveis 1 a 70) ---
    // Blue Gate (Nível Mínimo 1)
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 1, tier_max: 5, monster_id: "rato-ruina", monster_name: "Rato da Ruína", boss_name: "Rato Gigante", difficulty: "normal", area_level: 5, required_level: 1 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 6, tier_max: 10, monster_id: "morcego", monster_name: "Morcego Sombrio", boss_name: "Morcego Ancestral", difficulty: "normal", area_level: 10, required_level: 6 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 11, tier_max: 15, monster_id: "goblin", monster_name: "Goblin", boss_name: "Goblin Chefão", difficulty: "normal", area_level: 15, required_level: 11 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 16, tier_max: 20, monster_id: "esqueleto-guerreiro", monster_name: "Esqueleto Guerreiro", boss_name: "Esqueleto Campeão", difficulty: "normal", area_level: 20, required_level: 16 },

    // Castelo dos Demônios (Nível Mínimo 10)
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 21, tier_max: 25, monster_id: "orc-jovem", monster_name: "Orc Jovem", boss_name: "Orc Senhor da Guerra", difficulty: "normal", area_level: 25, required_level: 21 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 26, tier_max: 30, monster_id: "zumbi-faminto", monster_name: "Zumbi Faminto", boss_name: "Zumbi Devorador", difficulty: "normal", area_level: 30, required_level: 26 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 31, tier_max: 35, monster_id: "slime-corrosivo", monster_name: "Slime Corrosivo", boss_name: "Slime Colossal", difficulty: "normal", area_level: 35, required_level: 31 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 36, tier_max: 40, monster_id: "aranha-pedra", monster_name: "Aranha de Pedra", boss_name: "Aranha Matriarca", difficulty: "normal", area_level: 40, required_level: 36 },

    // Ilha Jeju (Nível Mínimo 15)
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 41, tier_max: 45, monster_id: "golem-barro", monster_name: "Golem de Barro", boss_name: "Golem Ancestral", difficulty: "normal", area_level: 45, required_level: 41 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 46, tier_max: 50, monster_id: "espirito-errante", monster_name: "Espírito Errante", boss_name: "Espírito Carrasco", difficulty: "normal", area_level: 50, required_level: 46 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 51, tier_max: 55, monster_id: "cavaleiro-morto", monster_name: "Cavaleiro Morto", boss_name: "Comandante Caído", difficulty: "normal", area_level: 55, required_level: 51 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 56, tier_max: 60, monster_id: "mago-esqueleto", monster_name: "Mago Esqueleto", boss_name: "Necromante Supremo", difficulty: "normal", area_level: 60, required_level: 56 },

    // Templo de Carneton (Nível Mínimo 25)
    HuntTier { gate_id: "templo-carneton", gate_name: "Templo de Carneton", tier_min: 61, tier_max: 65, monster_id: "senhor-vampiro", monster_name: "Senhor Vampiro", boss_name: "Lorde Vampiro", difficulty: "normal", area_level: 65, required_level: 61 },
    HuntTier { gate_id: "templo-carneton", gate_name: "Templo de Carneton", tier_min: 66, tier_max: 70, monster_id: "troll-cavernas", monster_name: "Troll das Cavernas", boss_name: "Troll Ancião", difficulty: "normal", area_level: 70, required_level: 66 },

    // --- Dificuldade Difícil (Níveis 71 a 140) ---
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 1, tier_max: 5, monster_id: "rato-ruina", monster_name: "Rato da Ruína", boss_name: "", difficulty: "dificil", area_level: 75, required_level: 71 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 6, tier_max: 10, monster_id: "morcego", monster_name: "Morcego Sombrio", boss_name: "", difficulty: "dificil", area_level: 80, required_level: 76 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 11, tier_max: 15, monster_id: "goblin", monster_name: "Goblin", boss_name: "", difficulty: "dificil", area_level: 85, required_level: 81 },
    HuntTier { gate_id: "blue-gate", gate_name: "Blue Gate", tier_min: 16, tier_max: 20, monster_id: "esqueleto-guerreiro", monster_name: "Esqueleto Guerreiro", boss_name: "", difficulty: "dificil", area_level: 90, required_level: 86 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 21, tier_max: 25, monster_id: "orc-jovem", monster_name: "Orc Jovem", boss_name: "", difficulty: "dificil", area_level: 95, required_level: 91 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 26, tier_max: 30, monster_id: "zumbi-faminto", monster_name: "Zumbi Faminto", boss_name: "", difficulty: "dificil", area_level: 100, required_level: 96 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 31, tier_max: 35, monster_id: "slime-corrosivo", monster_name: "Slime Corrosivo", boss_name: "", difficulty: "dificil", area_level: 105, required_level: 101 },
    HuntTier { gate_id: "castelo-demonios", gate_name: "Castelo dos Demônios", tier_min: 36, tier_max: 40, monster_id: "aranha-pedra", monster_name: "Aranha de Pedra", boss_name: "", difficulty: "dificil", area_level: 110, required_level: 106 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 41, tier_max: 45, monster_id: "golem-barro", monster_name: "Golem de Barro", boss_name: "", difficulty: "dificil", area_level: 115, required_level: 111 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 46, tier_max: 50, monster_id: "espirito-errante", monster_name: "Espírito Errante", boss_name: "", difficulty: "dificil", area_level: 120, required_level: 116 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 51, tier_max: 55, monster_id: "cavaleiro-morto", monster_name: "Cavaleiro Morto", boss_name: "", difficulty: "dificil", area_level: 125, required_level: 121 },
    HuntTier { gate_id: "ilha-jeju", gate_name: "Ilha Jeju", tier_min: 56, tier_max: 60, monster_id: "mago-esqueleto", monster_name: "Mago Esqueleto", boss_name: "", difficulty: "dificil", area_level: 130, required_level: 126 },
    HuntTier { gate_id: "templo-carneton", gate_name: "Templo de Carneton", tier_min: 61, tier_max: 65, monster_id: "senhor-vampiro", monster_name: "Senhor Vampiro", boss_name: "", difficulty: "dificil", area_level: 135, required_level: 131 },
    HuntTier { gate_id: "templo-carneton", gate_name: "Templo de Carneton", tier_min: 66, tier_max: 70, monster_id: "troll-cavernas", monster_name: "Troll das Cavernas", boss_name: "", difficulty: "dificil", area_level: 140, required_level: 136 },
];

pub fn get_best_hunt_for_level(level: u32) -> &'static HuntTier {
    if level == 0 {
        return &CATALOG_TIERS[0];
    }
    let mut best = &CATALOG_TIERS[0];
    let mut found = false;
    for tier in CATALOG_TIERS {
        if level >= tier.required_level
            && (!found || tier.area_level > best.area_level) {
                best = tier;
                found = true;
            }
    }
    best
}

pub fn get_hunt_name(gate_id: &str, tier_min: u32) -> String {
    for tier in CATALOG_TIERS {
        if tier.gate_id == gate_id && tier.tier_min == tier_min {
            return format!("{} ({} Nv {})", tier.gate_name, tier.monster_name, tier.area_level);
        }
    }
    if !gate_id.is_empty() {
        return format!("{gate_id} [T{tier_min}]");
    }
    "Desconhecido".to_string()
}

pub fn get_gate_for_tier(tier_min: u32) -> &'static str {
    for tier in CATALOG_TIERS {
        if tier.tier_min == tier_min {
            return tier.gate_id;
        }
    }
    "blue-gate"
}

pub fn get_next_codex_hunt_for_level(
    level: u32,
    codex_kills: &HashMap<String, u64>,
    target_milestone: u64,
    locked_hunts: &HashMap<String, Instant>,
) -> (&'static HuntTier, bool, usize, usize) {
    if level == 0 {
        return (&CATALOG_TIERS[0], false, 0, 0);
    }
    let mut total_monsters = 0;
    let mut completed_monsters = 0;
    let mut pending_tier: Option<&'static HuntTier> = None;
    let now = Instant::now();

    for tier in CATALOG_TIERS {
        if tier.difficulty == "normal" && level >= tier.required_level {
            if level >= 25 && tier.area_level + 10 < level {
                continue;
            }
            total_monsters += 1;
            let kills = codex_kills.get(tier.monster_id).copied().unwrap_or(0);
            let tier_key = format!("{}:{}:{}", tier.gate_id, tier.tier_min, tier.difficulty);
            let is_locked = locked_hunts.get(&tier_key).is_some_and(|exp| *exp > now);

            if kills >= target_milestone {
                completed_monsters += 1;
            } else if pending_tier.is_none() && !is_locked {
                pending_tier = Some(tier);
            }
        }
    }

    if pending_tier.is_none() && level >= 71 {
        for tier in CATALOG_TIERS {
            if tier.difficulty == "dificil" && level >= tier.required_level {
                total_monsters += 1;
                let kills = codex_kills.get(tier.monster_id).copied().unwrap_or(0);
                let tier_key = format!("{}:{}:{}", tier.gate_id, tier.tier_min, tier.difficulty);
                let is_locked = locked_hunts.get(&tier_key).is_some_and(|exp| *exp > now);

                if kills >= target_milestone {
                    completed_monsters += 1;
                } else if pending_tier.is_none() && !is_locked {
                    pending_tier = Some(tier);
                }
            }
        }
    }

    if let Some(tier) = pending_tier {
        (tier, true, completed_monsters, total_monsters)
    } else {
        (get_best_hunt_for_level(level), false, completed_monsters, total_monsters)
    }
}
