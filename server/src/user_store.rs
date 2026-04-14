use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::sync::RwLock;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub email:      String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum UserEvent {
    #[serde(rename = "user_created")]
    Created { email: String, created_at: String },
    #[serde(rename = "quest_saved")]
    QuestSaved { email: String, quest_id: String, at: String },
    #[serde(rename = "quest_unsaved")]
    QuestUnsaved { email: String, quest_id: String, at: String },
}

/// Persisted store for user profiles and saved-quest lists.
/// User events are append-only JSONL; state is rebuilt on startup.
pub struct UserStore {
    users:        RwLock<HashMap<String, UserProfile>>,
    saved_quests: RwLock<HashMap<String, Vec<String>>>,
    file_path:    String,
}

impl UserStore {
    pub fn load(file_path: &str) -> Self {
        let mut users:        HashMap<String, UserProfile> = HashMap::new();
        let mut saved_quests: HashMap<String, Vec<String>> = HashMap::new();

        if let Ok(f) = std::fs::File::open(file_path) {
            for line in std::io::BufReader::new(f).lines().flatten() {
                if let Ok(ev) = serde_json::from_str::<UserEvent>(&line) {
                    match ev {
                        UserEvent::Created { email, created_at } => {
                            users.insert(email.clone(), UserProfile { email, created_at });
                        }
                        UserEvent::QuestSaved { email, quest_id, .. } => {
                            let v = saved_quests.entry(email).or_default();
                            if !v.contains(&quest_id) { v.push(quest_id); }
                        }
                        UserEvent::QuestUnsaved { email, quest_id, .. } => {
                            if let Some(v) = saved_quests.get_mut(&email) {
                                v.retain(|q| q != &quest_id);
                            }
                        }
                    }
                }
            }
        }

        UserStore {
            users:        RwLock::new(users),
            saved_quests: RwLock::new(saved_quests),
            file_path:    file_path.to_string(),
        }
    }

    /// Return existing profile or create a new one.
    pub fn get_or_create(&self, email: &str) -> UserProfile {
        let key = email.to_lowercase();
        if let Some(p) = self.users.read().unwrap().get(&key).cloned() {
            return p;
        }
        let now = Utc::now().to_rfc3339();
        let profile = UserProfile { email: key.clone(), created_at: now.clone() };
        self.users.write().unwrap().insert(key.clone(), profile.clone());
        self.append(&UserEvent::Created { email: key, created_at: now });
        profile
    }

    pub fn get(&self, email: &str) -> Option<UserProfile> {
        self.users.read().unwrap().get(&email.to_lowercase()).cloned()
    }

    pub fn save_quest(&self, email: &str, quest_id: &str) {
        let key = email.to_lowercase();
        let qid = quest_id.to_string();
        let mut map = self.saved_quests.write().unwrap();
        let list = map.entry(key.clone()).or_default();
        if list.contains(&qid) { return; }
        list.push(qid.clone());
        drop(map);
        self.append(&UserEvent::QuestSaved { email: key, quest_id: qid, at: Utc::now().to_rfc3339() });
    }

    pub fn unsave_quest(&self, email: &str, quest_id: &str) {
        let key = email.to_lowercase();
        let qid = quest_id.to_string();
        {
            let mut map = self.saved_quests.write().unwrap();
            if let Some(v) = map.get_mut(&key) { v.retain(|q| q != &qid); }
        }
        self.append(&UserEvent::QuestUnsaved { email: key, quest_id: qid, at: Utc::now().to_rfc3339() });
    }

    pub fn saved_quest_ids(&self, email: &str) -> Vec<String> {
        self.saved_quests.read().unwrap()
            .get(&email.to_lowercase()).cloned().unwrap_or_default()
    }

    fn append(&self, ev: &UserEvent) {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&self.file_path) {
            if let Ok(line) = serde_json::to_string(ev) {
                let _ = writeln!(f, "{line}");
            }
        }
    }
}
