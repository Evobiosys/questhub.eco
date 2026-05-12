mod connect_store;
mod email;
mod handlers;
mod index;
mod routes;
mod session_store;
mod spam;
mod state;
mod user_store;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use kidur_log::MutationLog;
use kidur_supertag::SupertagRegistry;
use tokio::sync::broadcast;
use tracing_subscriber::EnvFilter;

use crate::connect_store::ConnectionStore;
use crate::index::Index;
use crate::session_store::SessionStore;
use crate::spam::SpamGuard;
use crate::state::QuestHubState;
use crate::user_store::UserStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Paths from env or defaults
    let data_dir = PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "data".into()));
    let supertags_dir =
        PathBuf::from(std::env::var("SUPERTAGS_DIR").unwrap_or_else(|_| "supertags".into()));
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".into());
    let listen_addr: SocketAddr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;

    // Ensure data dir exists
    std::fs::create_dir_all(&data_dir)?;

    let log_path = data_dir.join("kidur.jsonl");

    // 1. Open mutation log
    let mut log = MutationLog::open(&log_path)?;
    tracing::info!("Mutation log opened at {}", log_path.display());

    // 2. Build in-memory index from log
    let mut idx = Index::from_log(&log_path)?;
    let quest_count = idx.quest_count();
    tracing::info!("Index rebuilt: {} quests loaded", quest_count);

    // 3. Load seed data if index is empty
    if quest_count == 0 {
        let seed_path = PathBuf::from("seeds/questhub-quests.jsonl");
        if seed_path.exists() {
            tracing::info!("Loading seed data from {}", seed_path.display());
            let seed_entries = MutationLog::replay(&seed_path)?;
            for entry in seed_entries {
                log.append(entry.mutation.clone())?;
                idx.apply_mutation(entry.mutation);
            }
            tracing::info!("Loaded {} seed quests", idx.quest_count());
        } else {
            tracing::warn!("No seed data found at {}", seed_path.display());
        }
    }

    // 4. Load supertag registry
    let registry = SupertagRegistry::from_dir(&supertags_dir)?;
    tracing::info!("Supertag registry loaded: {:?}", registry.names());

    // 5. Create broadcast channel for SSE
    let (quest_tx, _) = broadcast::channel(256);

    // 6. Connection store
    let connect_store = Arc::new(ConnectionStore::load(
        data_dir.join("connections.jsonl").to_str().unwrap(),
    ));

    // 7. Email config (optional)
    let email_config = email::EmailConfig::from_env();
    if email_config.is_none() {
        tracing::warn!("SMTP not configured — emails disabled. Set SMTP_HOST, SMTP_PORT, SMTP_USER, SMTP_PASS");
    }

    // 8. Session store (in-memory — users re-authenticate after restart)
    let session_store = Arc::new(SessionStore::new());

    // 9. User store (persisted to data/users.jsonl)
    let user_store = Arc::new(UserStore::load(
        data_dir.join("users.jsonl").to_str().unwrap(),
    ));
    tracing::info!("User store loaded");

    // 10. Spam guard (in-memory: rate limit + PoW captcha + time-trap)
    let spam_guard = Arc::new(SpamGuard::new());

    // 11. Build state
    let state = QuestHubState {
        log: Arc::new(Mutex::new(log)),
        index: Arc::new(RwLock::new(idx)),
        registry: Arc::new(registry),
        quest_tx,
        connect_store,
        email_config,
        session_store,
        user_store,
        spam_guard,
    };

    // 11. Build router and serve
    let app = routes::build_router(state, &static_dir);

    tracing::info!("QuestHub listening on {listen_addr}");
    let listener = tokio::net::TcpListener::bind(listen_addr).await?;
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
