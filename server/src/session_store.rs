use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const MAGIC_LINK_TTL: u64    = 900;          // 15 minutes
const SESSION_TTL:    u64    = 30 * 86_400;  // 30 days

struct MagicLink {
    email:      String,
    created_at: u64,
}

struct SessionRecord {
    email:      String,
    created_at: u64,
}

/// In-memory store for magic-link tokens and active sessions.
/// Not persisted — users re-authenticate after a server restart.
pub struct SessionStore {
    magic_links: RwLock<HashMap<String, MagicLink>>,
    sessions:    RwLock<HashMap<String, SessionRecord>>,
}

impl SessionStore {
    pub fn new() -> Self {
        SessionStore {
            magic_links: RwLock::new(HashMap::new()),
            sessions:    RwLock::new(HashMap::new()),
        }
    }

    /// Generate a one-time magic-link token for the given email.
    pub fn create_magic_link(&self, email: &str) -> String {
        let tok = new_token();
        let now = unix_now();
        let mut map = self.magic_links.write().unwrap();
        map.retain(|_, v| now - v.created_at < MAGIC_LINK_TTL);
        map.insert(tok.clone(), MagicLink { email: email.to_lowercase(), created_at: now });
        tok
    }

    /// Verify a magic-link token (one-time use). Returns email if valid and not expired.
    pub fn verify_magic_link(&self, token: &str) -> Option<String> {
        let now = unix_now();
        let mut map = self.magic_links.write().unwrap();
        let rec = map.remove(token)?;
        if now - rec.created_at > MAGIC_LINK_TTL { return None; }
        Some(rec.email)
    }

    /// Create a persistent session for the given email. Returns the session token.
    pub fn create_session(&self, email: &str) -> String {
        let tok = new_token();
        self.sessions.write().unwrap().insert(tok.clone(), SessionRecord {
            email:      email.to_lowercase(),
            created_at: unix_now(),
        });
        tok
    }

    /// Resolve a session token to an email. Returns None if missing or expired.
    pub fn get_session_email(&self, token: &str) -> Option<String> {
        let now = unix_now();
        let map = self.sessions.read().unwrap();
        let s   = map.get(token)?;
        if now - s.created_at > SESSION_TTL { return None; }
        Some(s.email.clone())
    }

    /// Invalidate a session (sign out).
    pub fn invalidate(&self, token: &str) {
        self.sessions.write().unwrap().remove(token);
    }
}

fn new_token() -> String {
    Uuid::new_v4().to_string().replace('-', "")
}

fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO).as_secs()
}

/// Extract the `qh_session` cookie value from Axum headers.
pub fn get_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    let val = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in val.split(';') {
        if let Some(tok) = part.trim().strip_prefix("qh_session=") {
            return Some(tok.to_string());
        }
    }
    None
}
