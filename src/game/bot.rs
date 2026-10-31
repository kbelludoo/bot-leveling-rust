use crate::engine::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::engine::stamina::{StaminaConfig, StaminaController};
use crate::game::catalog::*;
use crate::game::party::GLOBAL_PARTY;
use crate::game::proto::*;
use crate::game::types::{AccountConfig, CharacterStatus};
use crate::network::socket::LevelingSocketClient;
use crate::subsystem::dailies::DailyManager;
use crate::subsystem::inventory::InventoryManager;
use crate::subsystem::potions::PotionManager;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::info;

pub struct BotInstance {
    pub config: AccountConfig,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub stamina: Arc<StaminaController>,
    pub inventory: Arc<InventoryManager>,
    pub dailies: Arc<DailyManager>,
    pub potions: Arc<PotionManager>,
    pub socket: Arc<LevelingSocketClient>,

    pub status: Arc<Mutex<CharacterStatus>>,
    pub req_counter: AtomicU64,
    pub running: AtomicBool,

    // Internal State
    pub in_game: AtomicBool,
    pub hunt_confirmed: AtomicBool,
    pub current_gate: Mutex<String>,
    pub current_tier: AtomicU32,
    pub current_difficulty: Mutex<String>,
    pub party_min_level: AtomicU32,
    pub last_attempted_hunt_key: Mutex<String>,
    pub codex_kills: Mutex<HashMap<String, u64>>,
    pub active_party_members: Mutex<HashMap<String, bool>>,

    pub stat_points: AtomicU32,
    pub talent_points: AtomicU32,
    pub promotion_locked: AtomicBool,
    pub level_cap: AtomicU32,
    pub banked_xp: AtomicU64,
    pub automation_configured: AtomicBool,

    // Timers
    pub last_hunt_check: Mutex<Instant>,
    pub last_equip_check: Mutex<Instant>,
    pub last_repair_check: Mutex<Instant>,
    pub last_mission_check: Mutex<Instant>,
    pub last_shadow_check: Mutex<Instant>,
    pub last_promotion_check: Mutex<Instant>,
    pub last_gem_check: Mutex<Instant>,
    pub last_enchant_check: Mutex<Instant>,
    pub last_shadow_evolve_check: Mutex<Instant>,
    pub last_expedition_check: Mutex<Instant>,
    pub last_fatigue_log: Mutex<Instant>,

    // Fatigue
    pub fatigue_milli: AtomicU32,
    pub farm_available_ms: AtomicU64,
    pub online_kpm: Mutex<f64>,
    pub is_resting: AtomicBool,
    pub rest_start_time: Mutex<Instant>,

    // Exp stats
    pub xp_start_time: Mutex<Option<Instant>>,
    pub last_level: AtomicU32,
    pub last_exp: AtomicU64,
    pub total_xp_gained: AtomicU64,
}

impl BotInstance {
    pub fn new(config: AccountConfig) -> Self {
        let cb_cfg = CircuitBreakerConfig::default();
        let cb = Arc::new(CircuitBreaker::new(&config.id, cb_cfg));
        let stam = Arc::new(StaminaController::new(StaminaConfig::default()));

        let device_uuid = if !config.credentials.device_uuid.is_empty() {
            config.credentials.device_uuid.clone()
        } else {
            uuid::Uuid::new_v4().to_string()
        };

        let ws_url = if config.credentials.origin.is_empty() {
            format!("wss://levelingidle2d.com.br/ws?device={}", device_uuid)
        } else {
            let orig = &config.credentials.origin;
            let scheme = if orig.starts_with("https") { "wss" } else { "ws" };
            let host = orig.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
            format!("{}://{}/ws?device={}", scheme, host, device_uuid)
        };

        let socket = Arc::new(LevelingSocketClient::new(&ws_url));
        let now = Instant::now();

        let st = CharacterStatus {
            name: config.name.clone(),
            current_area: "Blue Gate (Rato da Ruína Nv 5)".to_string(),
            status: "Iniciando...".to_string(),
            ..Default::default()
        };

        Self {
            config,
            circuit_breaker: cb,
            stamina: stam,
            inventory: Arc::new(InventoryManager::default()),
            dailies: Arc::new(DailyManager::default()),
            potions: Arc::new(PotionManager::default()),
            socket,
            status: Arc::new(Mutex::new(st)),
            req_counter: AtomicU64::new(1),
            running: AtomicBool::new(false),

            in_game: AtomicBool::new(false),
            hunt_confirmed: AtomicBool::new(false),
            current_gate: Mutex::new(String::new()),
            current_tier: AtomicU32::new(0),
            current_difficulty: Mutex::new("normal".to_string()),
            party_min_level: AtomicU32::new(1),
            last_attempted_hunt_key: Mutex::new(String::new()),
            codex_kills: Mutex::new(HashMap::new()),
            active_party_members: Mutex::new(HashMap::new()),

            stat_points: AtomicU32::new(0),
            talent_points: AtomicU32::new(0),
            promotion_locked: AtomicBool::new(false),
            level_cap: AtomicU32::new(100),
            banked_xp: AtomicU64::new(0),
            automation_configured: AtomicBool::new(false),

            last_hunt_check: Mutex::new(now),
            last_equip_check: Mutex::new(now),
            last_repair_check: Mutex::new(now),
            last_mission_check: Mutex::new(now),
            last_shadow_check: Mutex::new(now),
            last_promotion_check: Mutex::new(now),
            last_gem_check: Mutex::new(now),
            last_enchant_check: Mutex::new(now),
            last_shadow_evolve_check: Mutex::new(now),
            last_expedition_check: Mutex::new(now),
            last_fatigue_log: Mutex::new(now),

            fatigue_milli: AtomicU32::new(0),
            farm_available_ms: AtomicU64::new(0),
            online_kpm: Mutex::new(0.0),
            is_resting: AtomicBool::new(false),
            rest_start_time: Mutex::new(now),

            xp_start_time: Mutex::new(None),
            last_level: AtomicU32::new(0),
            last_exp: AtomicU64::new(0),
            total_xp_gained: AtomicU64::new(0),
        }
    }

    pub fn next_req_id(&self) -> u64 {
        self.req_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn add_log(&self, msg: String) {
        let mut st = self.status.lock().await;
        st.logs.push(msg);
        if st.logs.len() > 15 {
            st.logs.remove(0);
        }
    }

    pub async fn send_auth(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let user = if !self.config.credentials.username.is_empty() {
            &self.config.credentials.username
        } else {
            &self.config.credentials.email
        };
        let packet = build_auth_login(req_id, user, &self.config.credentials.password);
        self.socket.send_binary(packet).await
    }

    pub async fn send_ping(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_ping(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_start_hunt(&self, gate_id: &str, tier_min: u32, difficulty: &str) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_start_hunt(req_id, gate_id, tier_min, difficulty);
        *self.last_attempted_hunt_key.lock().await = format!("{}:{}:{}", gate_id, tier_min, difficulty);
        info!("[{}] Enviando StartHunt (gate={}, tier={}, dif={})", self.config.id, gate_id, tier_min, difficulty);
        self.socket.send_binary(packet).await
    }

    pub async fn send_leave_hunt(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_leave_hunt(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_give_up(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_give_up(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_equip_best(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_equip_best(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_repair_all(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_repair_all(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_claim_missions(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_claim_missions(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_set_automation(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let lvl = self.status.lock().await.level;
        let packet = build_set_automation(req_id, lvl);
        self.socket.send_binary(packet).await
    }

    pub async fn send_party_invite(&self, target_name: &str) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_party_invite(req_id, target_name);
        info!("[{}] Enviando convite de Party para: {}", self.config.id, target_name);
        self.socket.send_binary(packet).await
    }

    pub async fn send_party_answer(&self, from_account: i64, accept: bool) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_party_answer(req_id, from_account, accept);
        self.socket.send_binary(packet).await
    }

    pub async fn send_shadow_team_best(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_shadow_team_best(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_shadow_equip_best(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_shadow_equip_best(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_shadow_expedition(&self, claim: bool, start_best: bool) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_shadow_expedition(req_id, claim, start_best);
        self.socket.send_binary(packet).await
    }

    pub async fn send_start_promotion(&self) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_start_promotion(req_id);
        self.socket.send_binary(packet).await
    }

    pub async fn send_combine_gems(&self, tier: u32) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_combine_gems(req_id, tier);
        self.socket.send_binary(packet).await
    }

    pub async fn send_enchant(&self, slot: u32, safe_only: bool) -> Result<(), String> {
        let req_id = self.next_req_id();
        let packet = build_enchant(req_id, slot, safe_only);
        self.socket.send_binary(packet).await
    }

    pub async fn send_allocate_stats(&self, points: u32) -> Result<(), String> {
        if points == 0 {
            return Ok(());
        }
        let voc = {
            let st = self.status.lock().await;
            if !st.vocation.is_empty() {
                st.vocation.clone()
            } else {
                self.config.settings.preferred_class.clone()
            }
        };

        let mut stats = Vec::new();
        match voc.as_str() {
            "mago" => {
                let int_pts = (points as f64 * 0.8) as u32;
                let vit_pts = points - int_pts;
                if int_pts > 0 { stats.push((6, int_pts)); }
                if vit_pts > 0 { stats.push((3, vit_pts)); }
            }
            "healer" | "sacerdote" => {
                let int_pts = (points as f64 * 0.7) as u32;
                let vit_pts = points - int_pts;
                if int_pts > 0 { stats.push((6, int_pts)); }
                if vit_pts > 0 { stats.push((3, vit_pts)); }
            }
            "arqueiro" => {
                let agi_pts = (points as f64 * 0.7) as u32;
                let per_pts = points - agi_pts;
                if agi_pts > 0 { stats.push((2, agi_pts)); }
                if per_pts > 0 { stats.push((7, per_pts)); }
            }
            "assassino" => {
                let agi_pts = (points as f64 * 0.6) as u32;
                let per_pts = points - agi_pts;
                if agi_pts > 0 { stats.push((2, agi_pts)); }
                if per_pts > 0 { stats.push((7, per_pts)); }
            }
            _ => {
                let str_pts = (points as f64 * 0.8) as u32;
                let vit_pts = points - str_pts;
                if str_pts > 0 { stats.push((1, str_pts)); }
                if vit_pts > 0 { stats.push((3, vit_pts)); }
            }
        }

        let req_id = self.next_req_id();
        let packet = build_allocate_stats(req_id, &stats);
        self.socket.send_binary(packet).await
    }

    pub async fn handle_inbound_packet(&self, data: &[u8]) {
        let mut in_data = data;
        while !in_data.is_empty() {
            let (num, wtype, n) = match consume_tag(in_data) {
                Some(t) => t,
                None => break,
            };
            in_data = &in_data[n..];

            match wtype {
                WIRE_VARINT => {
                    let (_, n2) = match consume_varint(in_data) {
                        Some(v) => v,
                        None => break,
                    };
                    in_data = &in_data[n2..];
                }
                WIRE_BYTES => {
                    let (bytes_val, n2) = match consume_bytes(in_data) {
                        Some(b) => b,
                        None => break,
                    };
                    in_data = &in_data[n2..];

                    match num {
                        2 => { // Error
                            let err_msg = parse_string_field(bytes_val, 1);
                            if err_msg == "party.invite.in_party" {
                                self.add_log("Membro já está na Party.".to_string()).await;
                            } else if err_msg == "ratelimit.party" {
                                self.add_log("Party rate limit.".to_string()).await;
                            } else if err_msg == "shadow.notFound" || err_msg == "missions.unknown" || err_msg == "request.invalid" || err_msg == "repair.nothing" || err_msg == "content.unknown" || err_msg == "dungeon.locked" || err_msg == "item.notFound" || err_msg == "promotion.level" {
                                self.add_log(format!("Info servidor: {err_msg}")).await;
                            } else {
                                {
                                    let mut st = self.status.lock().await;
                                    st.status = format!("Erro: {err_msg}");
                                }
                                self.add_log(format!("Servidor erro: {err_msg}")).await;
                                let mut lock_key = self.last_attempted_hunt_key.lock().await;
                                if !lock_key.is_empty() && (err_msg == "level.tooLow" || err_msg == "gate.locked" || err_msg == "hunt.locked" || err_msg == "tier.invalid") {
                                    if let Ok(mut gp) = GLOBAL_PARTY.write() {
                                        gp.mark_hunt_locked(&lock_key, Duration::from_secs(900));
                                    }
                                    self.add_log(format!("Codex: Caçada {lock_key} bloqueada pelo servidor ({err_msg}). Avançando...")).await;
                                    lock_key.clear();
                                }
                            }
                        }
                        4 => { // AuthOk
                            let username = parse_string_field(bytes_val, 3);
                            self.in_game.store(true, Ordering::SeqCst);
                            {
                                let mut st = self.status.lock().await;
                                st.status = "Conectado e Autenticado".to_string();
                                if !username.is_empty() {
                                    st.name = username.clone();
                                }
                            }
                            self.add_log(format!("Login bem-sucedido! Bem-vindo, {username}.")).await;
                        }
                        6 => { // SessionEnded
                            self.hunt_confirmed.store(false, Ordering::SeqCst);
                            self.current_gate.lock().await.clear();
                            self.current_tier.store(0, Ordering::SeqCst);
                            self.add_log("Sessão finalizada pelo servidor.".to_string()).await;
                        }
                        8 => { // CharacterState
                            self.parse_character_state(bytes_val).await;
                        }
                        9 => { // CharacterPatch
                            self.parse_character_patch(bytes_val).await;
                        }
                        14 => { // WorldEvents
                            let mut st = self.status.lock().await;
                            st.kills += 1;
                            st.gold += 25;
                        }
                        30 => { // StartHuntResult
                            self.last_attempted_hunt_key.lock().await.clear();
                            self.hunt_confirmed.store(true, Ordering::SeqCst);
                            let gate_id = parse_string_field(bytes_val, 1);
                            let tier_min = parse_varint_field(bytes_val, 2) as u32;
                            let dif = parse_string_field(bytes_val, 3);
                            if !gate_id.is_empty() {
                                *self.current_gate.lock().await = gate_id.clone();
                                self.current_tier.store(tier_min, Ordering::SeqCst);
                                *self.current_difficulty.lock().await = dif;
                                let hunt_name = get_hunt_name(&gate_id, tier_min);
                                {
                                    let mut st = self.status.lock().await;
                                    st.current_area = hunt_name.clone();
                                    st.status = "hunting".to_string();
                                }
                                self.add_log(format!("Caçada iniciada: {hunt_name}")).await;
                            }
                        }
                        31 => { // LeaveHuntResult
                            self.hunt_confirmed.store(false, Ordering::SeqCst);
                            self.current_gate.lock().await.clear();
                            self.current_tier.store(0, Ordering::SeqCst);
                            self.add_log("Saiu da caçada.".to_string()).await;
                        }
                        36 => { // GiveUpResult
                            self.hunt_confirmed.store(false, Ordering::SeqCst);
                            self.current_gate.lock().await.clear();
                            self.current_tier.store(0, Ordering::SeqCst);
                            self.add_log("Desistência confirmada. Retomando...".to_string()).await;
                        }
                        41 => { // HuntSession
                            self.last_attempted_hunt_key.lock().await.clear();
                            let gate_id = parse_string_field(bytes_val, 1);
                            let tier_min = parse_varint_field(bytes_val, 2) as u32;
                            if !gate_id.is_empty() {
                                let mut st = self.status.lock().await;
                                st.current_area = get_hunt_name(&gate_id, tier_min);
                            }
                        }
                        43 => { // DeathState
                            self.hunt_confirmed.store(false, Ordering::SeqCst);
                            self.current_gate.lock().await.clear();
                            self.current_tier.store(0, Ordering::SeqCst);
                            self.add_log("Personagem derrotado em combate! Reiniciando...".to_string()).await;
                            let _ = self.send_give_up().await;
                        }
                        290 => { // PartyResponse
                            self.add_log("Atualização de Party recebida.".to_string()).await;
                        }
                        291 => { // PartyPush
                            self.handle_party_push(bytes_val).await;
                        }
                        _ => {}
                    }
                }
                _ => break,
            }
        }
    }

    async fn parse_character_state(&self, mut data: &[u8]) {
        while !data.is_empty() {
            let (num, wtype, n) = match consume_tag(data) {
                Some(t) => t,
                None => break,
            };
            data = &data[n..];

            if wtype == WIRE_BYTES {
                let (sub, n2) = match consume_bytes(data) {
                    Some(b) => b,
                    None => break,
                };
                data = &data[n2..];

                match num {
                    2 => { // Identity
                        let name = parse_string_field(sub, 1);
                        let cls = parse_string_field(sub, 6);
                        let mut st = self.status.lock().await;
                        if !name.is_empty() { st.name = name; }
                        if !cls.is_empty() { st.vocation = cls; }
                    }
                    3 => { // Progress
                        let level = parse_varint_field(sub, 1) as u32;
                        let exp = parse_varint_field(sub, 2);
                        let _xp_to_next = parse_varint_field(sub, 3);
                        let promo_locked = parse_varint_field(sub, 4) == 1;
                        let cap = parse_varint_field(sub, 5) as u32;
                        let banked = parse_varint_field(sub, 6);

                        if level > 0 {
                            let mut st = self.status.lock().await;
                            st.level = level;
                        }
                        self.last_level.store(level, Ordering::SeqCst);
                        self.last_exp.store(exp, Ordering::SeqCst);
                        self.promotion_locked.store(promo_locked, Ordering::SeqCst);
                        self.level_cap.store(cap, Ordering::SeqCst);
                        self.banked_xp.store(banked, Ordering::SeqCst);

                        {
                            let mut xp_start = self.xp_start_time.lock().await;
                            if xp_start.is_none() {
                                *xp_start = Some(Instant::now());
                            }
                        }

                        if promo_locked {
                            self.add_log(format!("⚡ Cap de Nível {cap} atingido! Promoção de Rank necessária.")).await;
                            let _ = self.send_start_promotion().await;
                        }
                    }
                    4 => { // Wallet
                        let gold = parse_varint_field(sub, 1) as i64;
                        if gold > 0 {
                            self.status.lock().await.gold = gold;
                            if let Ok(mut gp) = GLOBAL_PARTY.write() {
                                gp.report_gold(&self.config.id, gold);
                            }
                        }
                    }
                    5 => { // Points
                        self.stat_points.store(parse_varint_field(sub, 1) as u32, Ordering::SeqCst);
                        self.talent_points.store(parse_varint_field(sub, 2) as u32, Ordering::SeqCst);
                    }
                    6 => { // Stats
                        let mut st = self.status.lock().await;
                        st.hp = 200 + (st.level * 25);
                        st.max_hp = st.hp;
                        st.mana = 100 + (st.level * 15);
                        st.max_mana = st.mana;
                    }
                    14 => { // Hunt / HuntPrefs
                        let tier_min = parse_varint_field(sub, 1) as u32;
                        let area_str = parse_string_field(sub, 1);
                        let mut st = self.status.lock().await;
                        if tier_min > 0 {
                            st.current_area = get_hunt_name(get_gate_for_tier(tier_min), tier_min);
                        } else if !area_str.is_empty() {
                            st.current_area = area_str;
                        }
                    }
                    19 => { // Codex
                        let kills = parse_codex_monsters(sub);
                        let mut my_kills = self.codex_kills.lock().await;
                        for (k, v) in kills {
                            let curr = my_kills.entry(k).or_default();
                            if v > *curr { *curr = v; }
                        }
                    }
                    20 => { // FatigueState
                        self.parse_fatigue_state(sub).await;
                    }
                    _ => {}
                }
            } else {
                let (_, n2) = match consume_varint(data) {
                    Some(v) => v,
                    None => break,
                };
                data = &data[n2..];
            }
        }

        let lvl = {
            let mut st = self.status.lock().await;
            if st.status.is_empty() || st.status == "Conectado e Autenticado" {
                st.status = "hunting".to_string();
            }
            st.level
        };

        if lvl > 0 {
            let kills = self.codex_kills.lock().await.clone();
            if let Ok(mut gp) = GLOBAL_PARTY.write() {
                gp.report_level(&self.config.id, lvl);
                gp.report_kills(&self.config.id, &kills);
            }
        }
    }

    async fn parse_character_patch(&self, mut data: &[u8]) {
        while !data.is_empty() {
            let (num, wtype, n) = match consume_tag(data) {
                Some(t) => t,
                None => break,
            };
            data = &data[n..];

            if wtype == WIRE_BYTES {
                let (sub, n2) = match consume_bytes(data) {
                    Some(b) => b,
                    None => break,
                };
                data = &data[n2..];

                match num {
                    2 => { // Progress
                        let val = parse_varint_field(sub, 1) as u32;
                        let exp = parse_varint_field(sub, 2);
                        let xp_to_next = parse_varint_field(sub, 3);
                        let promo_locked = parse_varint_field(sub, 4) == 1;
                        let cap = parse_varint_field(sub, 5) as u32;
                        let banked = parse_varint_field(sub, 6);

                        let last_exp_val = self.last_exp.load(Ordering::Relaxed);
                        let last_lvl_val = self.last_level.load(Ordering::Relaxed);

                        if exp > last_exp_val || val > last_lvl_val {
                            let pct = if xp_to_next > 0 { (exp as f64 / xp_to_next as f64) * 100.0 } else { 0.0 };
                            info!("[{}] EXP PROGRESS: Lv {} | {} / {} ({:.1}%) | Cap: {} | PromoLocked: {}", self.config.id, val, exp, xp_to_next, pct, cap, promo_locked);
                        }

                        if val > 0 && val < 200 {
                            let mut st = self.status.lock().await;
                            st.level = val;
                            st.hp = 200 + (val * 25);
                            st.max_hp = st.hp;
                            st.mana = 100 + (val * 15);
                            st.max_mana = st.mana;
                        }

                        let mut xp_start = self.xp_start_time.lock().await;
                        if xp_start.is_none() {
                            *xp_start = Some(Instant::now());
                            self.last_level.store(val, Ordering::SeqCst);
                            self.last_exp.store(exp, Ordering::SeqCst);
                        } else {
                            let gain: i64 = if val > last_lvl_val {
                                self.add_log(format!("🎉 LEVEL UP! Nível {val} alcançado!")).await;
                                exp as i64 + 500
                            } else if exp > last_exp_val {
                                (exp - last_exp_val) as i64
                            } else {
                                0
                            };

                            if gain > 0 {
                                let total = self.total_xp_gained.fetch_add(gain as u64, Ordering::SeqCst) + gain as u64;
                                if let Some(st_time) = *xp_start {
                                    let elapsed = st_time.elapsed().as_secs_f64() / 3600.0;
                                    if elapsed > 0.001 {
                                        self.status.lock().await.exp_per_hour = total as f64 / elapsed;
                                    }
                                }
                            }
                            self.last_level.store(val, Ordering::SeqCst);
                            self.last_exp.store(exp, Ordering::SeqCst);
                        }

                        self.promotion_locked.store(promo_locked, Ordering::SeqCst);
                        self.level_cap.store(cap, Ordering::SeqCst);
                        self.banked_xp.store(banked, Ordering::SeqCst);

                        if promo_locked && self.last_promotion_check.lock().await.elapsed() > Duration::from_secs(30) {
                            *self.last_promotion_check.lock().await = Instant::now();
                            self.add_log(format!("⚡ Cap de Nível {cap} atingido! Promoção necessária.")).await;
                            let _ = self.send_start_promotion().await;
                        }
                    }
                    3 => { // Wallet
                        let gold = parse_varint_field(sub, 1) as i64;
                        if gold > 0 {
                            self.status.lock().await.gold = gold;
                            if let Ok(mut gp) = GLOBAL_PARTY.write() {
                                gp.report_gold(&self.config.id, gold);
                            }
                        }
                    }
                    4 => { // Points
                        self.stat_points.store(parse_varint_field(sub, 1) as u32, Ordering::SeqCst);
                        self.talent_points.store(parse_varint_field(sub, 2) as u32, Ordering::SeqCst);
                    }
                    16 | 17 => { // Codex / CodexKills
                        let kills = parse_codex_monsters(sub);
                        let mut my_kills = self.codex_kills.lock().await;
                        for (k, v) in kills {
                            let curr = my_kills.entry(k).or_default();
                            if v > *curr { *curr = v; }
                        }
                    }
                    18 => { // FatigueState
                        self.parse_fatigue_state(sub).await;
                    }
                    _ => {}
                }
            } else {
                let (_, n2) = match consume_varint(data) {
                    Some(v) => v,
                    None => break,
                };
                data = &data[n2..];
            }
        }

        {
            let mut st = self.status.lock().await;
            st.kills += 1;
            st.duration_sec += 3;
        }

        let curr_gate = self.current_gate.lock().await.clone();
        let curr_tier = self.current_tier.load(Ordering::Relaxed);
        if !curr_gate.is_empty() && curr_tier > 0 {
            for tier in CATALOG_TIERS {
                if tier.gate_id == curr_gate && tier.tier_min == curr_tier {
                    let mut my_kills = self.codex_kills.lock().await;
                    *my_kills.entry(tier.monster_id.to_string()).or_default() += 1;
                    break;
                }
            }
        }

        let lvl = self.status.lock().await.level;
        if lvl > 0 {
            let kills = self.codex_kills.lock().await.clone();
            if let Ok(mut gp) = GLOBAL_PARTY.write() {
                gp.report_level(&self.config.id, lvl);
                gp.report_kills(&self.config.id, &kills);
            }
        }
    }

    async fn handle_party_push(&self, mut data: &[u8]) {
        while !data.is_empty() {
            let (num, wtype, n) = match consume_tag(data) {
                Some(t) => t,
                None => break,
            };
            data = &data[n..];

            if wtype == WIRE_BYTES {
                let (sub, n2) = match consume_bytes(data) {
                    Some(b) => b,
                    None => break,
                };
                data = &data[n2..];

                match num {
                    1 => { // PartyState
                        self.add_log("Party sincronizada.".to_string()).await;
                        let mut p_data = sub;
                        let mut min_lvl = self.status.lock().await.level;
                        let mut leader_gate = String::new();
                        let mut leader_tier = 0u32;
                        let mut active_members = HashMap::new();

                        while !p_data.is_empty() {
                            let (p_num, p_type, pn) = match consume_tag(p_data) {
                                Some(t) => t,
                                None => break,
                            };
                            p_data = &p_data[pn..];

                            if p_type == WIRE_BYTES {
                                let (m_bytes, mn) = match consume_bytes(p_data) {
                                    Some(b) => b,
                                    None => break,
                                };
                                p_data = &p_data[mn..];

                                if p_num == 2 { // PartyMember
                                    let m_name = parse_string_field(m_bytes, 2);
                                    let m_lvl = parse_varint_field(m_bytes, 4) as u32;
                                    let is_leader = parse_varint_field(m_bytes, 6) == 1;
                                    let m_gate = parse_string_field(m_bytes, 18);
                                    let m_tier = parse_varint_field(m_bytes, 19) as u32;

                                    if !m_name.is_empty() {
                                        active_members.insert(m_name, true);
                                    }
                                    if m_lvl > 0 && m_lvl < min_lvl {
                                        min_lvl = m_lvl;
                                    }
                                    if is_leader && !m_gate.is_empty() {
                                        leader_gate = m_gate;
                                        leader_tier = m_tier;
                                    }
                                }
                            } else {
                                let (_, pn2) = match consume_varint(p_data) {
                                    Some(v) => v,
                                    None => break,
                                };
                                p_data = &p_data[pn2..];
                            }
                        }

                        *self.active_party_members.lock().await = active_members;
                        if min_lvl > 0 {
                            self.party_min_level.store(min_lvl, Ordering::SeqCst);
                        }

                        if !self.config.settings.party_leader && !leader_gate.is_empty() && leader_tier > 0 {
                            let current_g = self.current_gate.lock().await.clone();
                            let current_t = self.current_tier.load(Ordering::Relaxed);
                            let confirmed = self.hunt_confirmed.load(Ordering::Relaxed);

                            if !confirmed || current_g != leader_gate || current_t != leader_tier {
                                let _ = self.send_start_hunt(&leader_gate, leader_tier, "normal").await;
                                let area_name = get_hunt_name(&leader_gate, leader_tier);
                                self.status.lock().await.current_area = area_name.clone();
                                self.add_log(format!("Auto-Follow: Sincronizando caçada com o líder -> {area_name}")).await;
                            }
                        }
                    }
                    2 => { // PartyInvite
                        let from_acc = parse_varint_field(sub, 1) as i64;
                        let from_name = parse_string_field(sub, 2);
                        self.add_log(format!("Convite de Party recebido de {from_name} (#{from_acc})")).await;
                        if self.config.settings.auto_party {
                            let _ = self.send_party_answer(from_acc, true).await;
                            self.add_log(format!("Aceitou convite de Party de {from_name} automaticamente!")).await;
                        }
                    }
                    _ => {}
                }
            } else {
                let (_, n2) = match consume_varint(data) {
                    Some(v) => v,
                    None => break,
                };
                data = &data[n2..];
            }
        }
    }

    async fn parse_fatigue_state(&self, mut sub: &[u8]) {
        while !sub.is_empty() {
            let (num, wtype, n) = match consume_tag(sub) {
                Some(t) => t,
                None => break,
            };
            sub = &sub[n..];

            match wtype {
                WIRE_VARINT => {
                    let (v, n2) = match consume_varint(sub) {
                        Some(val) => val,
                        None => break,
                    };
                    sub = &sub[n2..];
                    match num {
                        1 => self.fatigue_milli.store(v as u32, Ordering::SeqCst),
                        4 => self.farm_available_ms.store(v, Ordering::SeqCst),
                        _ => {}
                    }
                }
                WIRE_FIXED64 => {
                    if sub.len() < 8 { break; }
                    let bytes: [u8; 8] = sub[..8].try_into().unwrap();
                    sub = &sub[8..];
                    if num == 5 {
                        *self.online_kpm.lock().await = f64::from_le_bytes(bytes);
                    }
                }
                WIRE_FIXED32 => {
                    if sub.len() < 4 { break; }
                    sub = &sub[4..];
                }
                WIRE_BYTES => {
                    let (_, n2) = match consume_bytes(sub) {
                        Some(b) => b,
                        None => break,
                    };
                    sub = &sub[n2..];
                }
                _ => break,
            }
        }

        let mut last_log = self.last_fatigue_log.lock().await;
        if last_log.elapsed() > Duration::from_secs(120) {
            *last_log = Instant::now();
            let fatigue_pct = self.fatigue_milli.load(Ordering::Relaxed) as f64 / 10.0;
            let farm_min = self.farm_available_ms.load(Ordering::Relaxed) as f64 / 60000.0;
            let kpm = *self.online_kpm.lock().await;
            info!("[{}] Fadiga: {:.1}% | Farm Disponível: {:.1} min | KPM: {:.1}", self.config.id, fatigue_pct, farm_min, kpm);
        }
    }

    pub async fn perform_tick(&self) {
        if !self.in_game.load(Ordering::Relaxed) {
            return;
        }

        {
            let mut st = self.status.lock().await;
            st.duration_sec += 3;
            if st.status.is_empty() {
                st.status = "hunting".to_string();
            }
        }

        let my_level = self.status.lock().await.level;
        if my_level > 0 {
            let kills = self.codex_kills.lock().await.clone();
            if let Ok(mut gp) = GLOBAL_PARTY.write() {
                gp.report_level(&self.config.id, my_level);
                gp.report_kills(&self.config.id, &kills);
            }
        }

        // 1. Fatigue & Rest
        let fatigue_milli = self.fatigue_milli.load(Ordering::Relaxed);
        let farm_ms = self.farm_available_ms.load(Ordering::Relaxed);
        let fatigue_high = fatigue_milli >= 800;
        let farm_exhausted = farm_ms > 0 && farm_ms < 120_000;
        let should_rest = fatigue_high || farm_exhausted;

        let fatigue_recovered = fatigue_milli < 300;
        let farm_recovered = farm_ms == 0 || farm_ms > 600_000;
        let can_resume = fatigue_recovered && farm_recovered;

        if should_rest && !self.is_resting.load(Ordering::Relaxed) {
            self.is_resting.store(true, Ordering::SeqCst);
            *self.rest_start_time.lock().await = Instant::now();

            if self.hunt_confirmed.load(Ordering::Relaxed) {
                let _ = self.send_leave_hunt().await;
                self.hunt_confirmed.store(false, Ordering::SeqCst);
                self.current_gate.lock().await.clear();
                self.current_tier.store(0, Ordering::SeqCst);
            }
            let mut st = self.status.lock().await;
            st.status = "resting".to_string();
            st.current_area = format!("Descansando (Fadiga: {:.0}%)", fatigue_milli as f64 / 10.0);
            self.add_log("💤 DESCANSO INICIADO por fadiga alta.".to_string()).await;
        }

        if self.is_resting.load(Ordering::Relaxed) {
            let elapsed = self.rest_start_time.lock().await.elapsed();
            if can_resume && elapsed > Duration::from_secs(300) {
                self.is_resting.store(false, Ordering::SeqCst);
                self.status.lock().await.status = "hunting".to_string();
                self.automation_configured.store(false, Ordering::SeqCst);
                self.add_log("⚡ DESCANSO CONCLUÍDO! Retomando caçada!".to_string()).await;
            } else {
                return;
            }
        }

        // 2. Auto-Hunt & Codex Coordination
        if self.config.settings.auto_hunt && my_level > 0 {
            if self.config.settings.party_leader || self.config.settings.party_members.is_empty() {
                let target_level = if let Ok(gp) = GLOBAL_PARTY.read() {
                    gp.get_min_level(my_level)
                } else {
                    my_level
                };

                let best: &HuntTier = if self.config.settings.codex_mode {
                    let target_milestone = if self.config.settings.codex_target == 0 { 2000 } else { self.config.settings.codex_target };
                    let (shared_kills, locked_hunts) = if let Ok(gp) = GLOBAL_PARTY.read() {
                        (gp.get_shared_codex_kills(), gp.get_locked_hunts())
                    } else {
                        (HashMap::new(), HashMap::new())
                    };

                    let (tier, pending, completed, total) = get_next_codex_hunt_for_level(target_level, &shared_kills, target_milestone, &locked_hunts);
                    let mut st = self.status.lock().await;
                    if pending {
                        let k = shared_kills.get(tier.monster_id).copied().unwrap_or(0);
                        st.codex_info = format!("{completed}/{total} mobs | {} ({k}/{target_milestone})", tier.monster_name);
                    } else {
                        st.codex_info = format!("Completo ({completed}/{total} mobs) | Foco XP");
                    }
                    tier
                } else {
                    let t = get_best_hunt_for_level(target_level);
                    self.status.lock().await.codex_info = "Foco XP Máximo (Rank Rush)".to_string();
                    t
                };

                if let Ok(mut gp) = GLOBAL_PARTY.write() {
                    gp.set_leader_hunt(best.gate_id, best.tier_min, best.difficulty);
                }

                let confirmed = self.hunt_confirmed.load(Ordering::Relaxed);
                let curr_g = self.current_gate.lock().await.clone();
                let curr_t = self.current_tier.load(Ordering::Relaxed);
                let curr_d = self.current_difficulty.lock().await.clone();

                if !confirmed || curr_g != best.gate_id || curr_t != best.tier_min || curr_d != best.difficulty {
                    let mut last_h = self.last_hunt_check.lock().await;
                    if last_h.elapsed() > Duration::from_secs(5) {
                        *last_h = Instant::now();
                        let _ = self.send_start_hunt(best.gate_id, best.tier_min, best.difficulty).await;
                        let h_name = get_hunt_name(best.gate_id, best.tier_min);
                        self.status.lock().await.current_area = h_name.clone();
                        self.add_log(format!("Codex Líder: Focando em {h_name}")).await;
                    }
                }
            } else {
                // Member follows leader
                let (target_gate, target_tier, target_dif) = if let Ok(gp) = GLOBAL_PARTY.read() {
                    gp.get_leader_hunt()
                } else {
                    (String::new(), 0, String::new())
                };

                if !target_gate.is_empty() && target_tier > 0 {
                    let confirmed = self.hunt_confirmed.load(Ordering::Relaxed);
                    let curr_g = self.current_gate.lock().await.clone();
                    let curr_t = self.current_tier.load(Ordering::Relaxed);
                    let curr_d = self.current_difficulty.lock().await.clone();

                    if !confirmed || curr_g != target_gate || curr_t != target_tier || curr_d != target_dif {
                        let mut last_h = self.last_hunt_check.lock().await;
                        if last_h.elapsed() > Duration::from_secs(4) {
                            *last_h = Instant::now();
                            let _ = self.send_start_hunt(&target_gate, target_tier, &target_dif).await;
                            let h_name = get_hunt_name(&target_gate, target_tier);
                            self.status.lock().await.current_area = h_name.clone();
                            self.add_log(format!("Auto-Follow: Sincronizando com líder -> {h_name}")).await;
                        }
                    }
                }
            }
        }

        // 3. Leader Party Invites
        if self.config.settings.party_leader && !self.config.settings.party_members.is_empty() {
            let dur = self.status.lock().await.duration_sec;
            if dur <= 15 || dur % 60 == 0 {
                let active_members = self.active_party_members.lock().await.clone();
                for member in &self.config.settings.party_members {
                    if !member.is_empty() && member != &self.config.credentials.username
                        && !active_members.get(member).copied().unwrap_or(false) {
                            let _ = self.send_party_invite(member).await;
                        }
                }
            }
        }

        // 4. Server Automations
        if !self.automation_configured.load(Ordering::Relaxed) {
            self.automation_configured.store(true, Ordering::SeqCst);
            let _ = self.send_set_automation().await;
        }

        // 5. Stat Points Distribution
        let pts = self.stat_points.swap(0, Ordering::SeqCst);
        if pts > 0 && self.config.settings.auto_stats {
            let _ = self.send_allocate_stats(pts).await;
        }

        // 6. Equip Best (30s)
        if self.config.settings.auto_equip && self.last_equip_check.lock().await.elapsed() > Duration::from_secs(30) {
            *self.last_equip_check.lock().await = Instant::now();
            let _ = self.send_equip_best().await;
        }

        // 7. Repair All (3 min)
        if self.config.settings.auto_repair && self.last_repair_check.lock().await.elapsed() > Duration::from_secs(180) {
            *self.last_repair_check.lock().await = Instant::now();
            let _ = self.send_repair_all().await;
        }

        // 8. Claim Missions (60s)
        if self.config.settings.auto_missions && self.last_mission_check.lock().await.elapsed() > Duration::from_secs(60) {
            *self.last_mission_check.lock().await = Instant::now();
            let _ = self.send_claim_missions().await;
        }

        // 9. Shadows (60s)
        if self.config.settings.auto_shadows && self.last_shadow_check.lock().await.elapsed() > Duration::from_secs(60) {
            *self.last_shadow_check.lock().await = Instant::now();
            let _ = self.send_shadow_team_best().await;
            let _ = self.send_shadow_equip_best().await;
        }

        // 10. Advanced Mechanics (Gems, Enchant, Expeditions)
        let gold = self.status.lock().await.gold;
        let reserve = self.config.settings.gold_reserve;

        if self.config.settings.auto_gems && gold >= reserve && self.last_gem_check.lock().await.elapsed() > Duration::from_secs(300) {
            *self.last_gem_check.lock().await = Instant::now();
            let _ = self.send_combine_gems(0).await;
        }

        if self.config.settings.auto_enchant && gold >= reserve && self.last_enchant_check.lock().await.elapsed() > Duration::from_secs(300) {
            *self.last_enchant_check.lock().await = Instant::now();
            for slot in 1..=6 {
                let _ = self.send_enchant(slot, true).await;
            }
        }

        if my_level >= 15 && self.last_expedition_check.lock().await.elapsed() > Duration::from_secs(900) {
            *self.last_expedition_check.lock().await = Instant::now();
            let _ = self.send_shadow_expedition(true, true).await;
        }
    }

    pub async fn run_loop(self: Arc<Self>) {
        self.running.store(true, Ordering::SeqCst);
        info!("🚀 Bot [{}] (Leveling Idle 100% Paridade - Native Rust Zero-GC) iniciado.", self.config.name);

        while self.running.load(Ordering::Relaxed) {
            {
                let mut st = self.status.lock().await;
                st.status = "Conectando ao servidor...".to_string();
            }

            let self_clone = self.clone();
            match self.socket.connect(move |bin| {
                let sc = self_clone.clone();
                tokio::spawn(async move {
                    sc.handle_inbound_packet(&bin).await;
                });
            }).await {
                Ok(_) => {
                    info!("[{}] Conexão WebSocket estabelecida com sucesso. Enviando login...", self.config.id);
                    {
                        let mut st = self.status.lock().await;
                        st.status = "Autenticando...".to_string();
                    }
                    sleep(Duration::from_millis(500)).await;
                    if let Err(e) = self.send_auth().await {
                        tracing::error!("[{}] Falha ao enviar auth: {}", self.config.id, e);
                    }

                    let mut ping_timer = tokio::time::interval(Duration::from_secs(10));
                    let mut tick_timer = tokio::time::interval(Duration::from_secs(3));

                    while self.running.load(Ordering::Relaxed) && self.socket.is_connected().await {
                        tokio::select! {
                            _ = ping_timer.tick() => {
                                let _ = self.send_ping().await;
                            }
                            _ = tick_timer.tick() => {
                                self.perform_tick().await;
                            }
                        }
                    }
                    tracing::warn!("[{}] Conexão perdida. Reconectando em 5 segundos...", self.config.id);
                }
                Err(e) => {
                    tracing::error!("[{}] Erro ao conectar WebSocket: {}. Reconectando em 5 segundos...", self.config.id, e);
                    {
                        let mut st = self.status.lock().await;
                        st.status = format!("Erro de conexão: {e}");
                    }
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}
