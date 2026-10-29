#[derive(Debug, Clone)]
pub struct AreaCandidate {
    pub id: &'static str,
    pub min_level: u32,
    pub max_level: u32,
    pub base_exp: f64,
    pub base_gold: f64,
}

pub static HUNTS_CATALOG: &[AreaCandidate] = &[
        AreaCandidate { id: "dungeon_e_rank", min_level: 1, max_level: 25, base_exp: 120.0, base_gold: 65.0 },
        AreaCandidate { id: "dungeon_d_rank", min_level: 25, max_level: 55, base_exp: 330.0, base_gold: 190.0 },
        AreaCandidate { id: "dungeon_c_rank", min_level: 55, max_level: 95, base_exp: 890.0, base_gold: 530.0 },
        AreaCandidate { id: "dungeon_b_rank", min_level: 95, max_level: 150, base_exp: 2300.0, base_gold: 1450.0 },
        AreaCandidate { id: "dungeon_a_rank", min_level: 150, max_level: 225, base_exp: 5800.0, base_gold: 3900.0 },
        AreaCandidate { id: "dungeon_s_rank", min_level: 225, max_level: 320, base_exp: 14500.0, base_gold: 10200.0 },
        AreaCandidate { id: "torre_dos_desafios", min_level: 320, max_level: 500, base_exp: 36000.0, base_gold: 26000.0 },
];

#[derive(Clone, Default)]
pub struct Catalog;

impl Catalog {
    pub fn select_best_hunt(&self, level: u32) -> &'static AreaCandidate {
        for area in HUNTS_CATALOG {
            if level >= area.min_level && level <= area.max_level {
                return area;
            }
        }
        &HUNTS_CATALOG[HUNTS_CATALOG.len() - 1]
    }
}
