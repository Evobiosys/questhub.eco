use std::sync::{Arc, Mutex, RwLock};
use kidur_log::MutationLog;
use kidur_supertag::SupertagRegistry;
use tokio::sync::broadcast;

use crate::connect_store::ConnectionStore;
use crate::email::EmailConfig;
use crate::index::Index;
use crate::session_store::SessionStore;
use crate::spam::SpamGuard;
use crate::user_store::UserStore;

/// Event pushed to SSE subscribers when a quest changes.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum QuestEvent {
    #[serde(rename = "quest_created")]
    Created { quest: crate::handlers::quest::QuestResponse },
}

/// Shared application state for all Axum handlers.
#[derive(Clone)]
pub struct QuestHubState {
    pub log:           Arc<Mutex<MutationLog>>,
    pub index:         Arc<RwLock<Index>>,
    pub registry:      Arc<SupertagRegistry>,
    pub quest_tx:      broadcast::Sender<QuestEvent>,
    pub connect_store: Arc<ConnectionStore>,
    pub email_config:  Option<EmailConfig>,
    pub session_store: Arc<SessionStore>,
    pub user_store:    Arc<UserStore>,
    pub spam_guard:    Arc<SpamGuard>,
}
