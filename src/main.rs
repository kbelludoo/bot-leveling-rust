#![allow(dead_code)]
#![allow(unused_imports)]

mod engine;
mod network;
mod subsystem;
mod game;
mod server;

use engine::supervisor::Supervisor;
use game::types::AccountConfig;
use server::dashboard::WebServer;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

const DEFAULT_ACCOUNTS_EXAMPLE: &str = include_str!("../accounts.example.json");

fn format_number(n: u64) -> String {
    if n < 1000 {
        format!("{n}")
    } else if n < 1_000_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        format!("{:.2}kk", n as f64 / 1_000_000.0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    println!("================================================================================");
    println!("       ⚡ LEVELING IDLE NATIVE RUST BOT (ZERO-GC & ALTA PERFORMANCE) ⚡");
    println!("================================================================================");

    let config_path = "accounts.json";
    let content = match std::fs::read_to_string(config_path) {
        Ok(data) => data,
        Err(_) => {
            warn!("Arquivo accounts.json não encontrado. Criando template padrão...");
            let _ = std::fs::write(config_path, DEFAULT_ACCOUNTS_EXAMPLE);
            DEFAULT_ACCOUNTS_EXAMPLE.to_string()
        }
    };

    let all_configs: Vec<AccountConfig> = match serde_json::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            error!("Falha crítica ao analisar accounts.json: {}", e);
            std::process::exit(1);
        }
    };

    let target_acc = std::env::var("ACCOUNT").unwrap_or_default();
    let mut active_configs = Vec::new();

    for a in all_configs {
        if !a.enabled && a.id != target_acc {
            continue;
        }
        if !target_acc.is_empty() && a.id != target_acc {
            continue;
        }
        active_configs.push(a);
    }

    if active_configs.is_empty() {
        warn!("Nenhuma conta com 'enabled: true' encontrada em accounts.json.");
        info!("Edite accounts.json e habilite sua conta para iniciar o bot.");
    }

    let supervisor = Arc::new(Supervisor::new(active_configs));
    supervisor.start_all().await;

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3003);

    let web_server = WebServer::new(supervisor.clone(), port);
    tokio::spawn(async move {
        if let Err(e) = web_server.run().await {
            error!("Erro no servidor web: {}", e);
        }
    });

    let sup_clone = supervisor.clone();
    tokio::spawn(async move {
        let mut tick = 0u64;
        loop {
            sleep(Duration::from_secs(4)).await;
            tick += 1;

            let mut total_kills = 0u64;
            let mut total_gold = 0i64;
            let active_bots = sup_clone.bots();
            let mut summaries = Vec::new();

            for b in active_bots {
                let st = b.status.lock().await.clone();
                total_kills += st.kills;
                total_gold += st.gold;
                summaries.push(format!("[{}] {} (Lv. {}) | Area: {} | Kills: {} | Status: {}", b.config.id, st.name, st.level, st.current_area, st.kills, st.status));
            }

            println!("================================================================================");
            println!("   ⚡ BOT ENGINE CONSOLE RUST ZERO-GC — Ticker #{} | Contas: {} | Ouro: {} | Kills: {}", tick, summaries.len(), format_number(total_gold.max(0) as u64), total_kills);
            println!("   🌐 Dashboard Web: http://localhost:{}", port);
            println!("================================================================================");
            for s in summaries {
                println!(" ▸ {}", s);
            }
        }
    });

    info!("Bot operacional na porta {}. Pressione Ctrl+C para encerrar.", port);
    signal::ctrl_c().await?;
    info!("Encerrando bot...");

    Ok(())
}
