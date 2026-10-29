use crate::engine::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::engine::stamina::{StaminaConfig, StaminaController};
use crate::game::proto::*;
use crate::game::types::{AccountConfig, CharacterStatus};
use crate::network::socket::LevelingSocketClient;
use crate::subsystem::dailies::DailyManager;
use crate::subsystem::inventory::InventoryManager;
use crate::subsystem::potions::PotionManager;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

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
    pub kills: AtomicU64,
    pub gold: AtomicU64,
    pub exp: AtomicU64,
    pub current_level: AtomicU32,
    pub running: AtomicBool,
}

impl BotInstance {
    pub fn new(config: AccountConfig) -> Self {
        let cb_cfg = CircuitBreakerConfig::default();
        let cb = Arc::new(CircuitBreaker::new(&config.id, cb_cfg));
        let stam = Arc::new(StaminaController::new(StaminaConfig::default()));

        let ws_url = if config.credentials.origin.is_empty() {
            "wss://api.leveling.idle/ws".to_string()
        } else {
            let orig = &config.credentials.origin;
            let scheme = if orig.starts_with("https") { "wss" } else { "ws" };
            let host = orig.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
            format!("{}://{}/ws", scheme, host)
        };

        let socket = Arc::new(LevelingSocketClient::new(&ws_url));

        Self {
            config,
            circuit_breaker: cb,
            stamina: stam,
            inventory: Arc::new(InventoryManager::default()),
            dailies: Arc::new(DailyManager::default()),
            potions: Arc::new(PotionManager::default()),
            socket,
            status: Arc::new(Mutex::new(CharacterStatus::default())),
            req_counter: AtomicU64::new(1),
            kills: AtomicU64::new(0),
            gold: AtomicU64::new(0),
            exp: AtomicU64::new(0),
            current_level: AtomicU32::new(1),
            running: AtomicBool::new(false),
        }
    }

    pub fn next_req_id(&self) -> u64 {
        self.req_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub async fn perform_automation_tick(&self) {
        if !self.socket.is_connected().await {
            return;
        }

        let req_id = self.next_req_id();

        // 1. Torre dos Desafios (Tower of Trials)
        if self.config.settings.auto_tower {
            info!("🗼 [Torre dos Desafios] Enviando avanço automático...");
            let packet = build_tower_request(req_id, true, true);
            let _ = self.socket.send_binary(packet).await;
        }

        // 2. Boss Diário
        if self.config.settings.auto_daily_boss {
            let req_id = self.next_req_id();
            info!("👑 [Boss Diário] Desafiando chefe diário...");
            let packet = build_daily_boss(req_id);
            let _ = self.socket.send_binary(packet).await;
        }

        // 3. Fusão de Gemas
        if self.config.settings.auto_gems {
            let req_id = self.next_req_id();
            let packet = build_combine_gems(req_id, 1);
            let _ = self.socket.send_binary(packet).await;
        }

        // 4. Encantamento de Equipamento
        if self.config.settings.auto_enchant {
            let req_id = self.next_req_id();
            let packet = build_enchant(req_id, 1, true);
            let _ = self.socket.send_binary(packet).await;
        }
    }

    pub async fn run_loop(self: Arc<Self>) {
        self.running.store(true, Ordering::SeqCst);
        info!("🚀 Bot [{}] (Leveling Idle - Protobuf) iniciado com sucesso.", self.config.name);

        let _ = self.socket.connect(|_| {}).await;

        let mut tick_timer = tokio::time::interval(Duration::from_secs(5));
        while self.running.load(Ordering::Relaxed) {
            tick_timer.tick().await;
            self.perform_automation_tick().await;
        }
    }
}
