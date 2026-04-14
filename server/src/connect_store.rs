use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConnectStatus {
    Pending,
    Approved,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub token: String,
    pub quest_id: String,
    pub quest_title: String,
    pub owner_contact: String,
    pub requester_name: String,
    pub requester_email: String,
    pub message: String,
    pub status: ConnectStatus,
    pub created_at: String,
}

pub struct ConnectionStore {
    requests: RwLock<HashMap<String, ConnectionRequest>>,
    file_path: String,
}

impl ConnectionStore {
    pub fn load(file_path: &str) -> Self {
        let mut map = HashMap::new();
        if let Ok(file) = std::fs::File::open(file_path) {
            for line in std::io::BufReader::new(file).lines().flatten() {
                if let Ok(req) = serde_json::from_str::<ConnectionRequest>(&line) {
                    map.insert(req.token.clone(), req);
                }
            }
        }
        Self {
            requests: RwLock::new(map),
            file_path: file_path.to_string(),
        }
    }

    pub fn insert(&self, req: ConnectionRequest) -> Result<(), String> {
        let json = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        {
            let mut map = self.requests.write().unwrap();
            map.insert(req.token.clone(), req);
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| e.to_string())?;
        writeln!(file, "{json}").map_err(|e| e.to_string())
    }

    pub fn get(&self, token: &str) -> Option<ConnectionRequest> {
        self.requests.read().unwrap().get(token).cloned()
    }

    /// Approve a pending request. Returns (request, newly_approved).
    /// If already approved, returns (request, false) without re-sending emails.
    pub fn approve(&self, token: &str) -> Option<(ConnectionRequest, bool)> {
        let mut map = self.requests.write().unwrap();
        let req = map.get_mut(token)?;
        let newly_approved = req.status == ConnectStatus::Pending;
        if newly_approved {
            req.status = ConnectStatus::Approved;
        }
        Some((req.clone(), newly_approved))
    }
}
