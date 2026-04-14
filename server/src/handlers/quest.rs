use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect, Response},
    Form,
};
use kidur_core::{FieldValue, Node, NodeId};
use kidur_log::Mutation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::session_store::get_session_cookie;
use crate::state::{QuestEvent, QuestHubState};

/// Public-facing quest representation.
/// Note: `contact` is intentionally absent — stored privately, never returned via API.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub category_custom: Option<String>,
    pub submitter_name: Option<String>,
    pub lifecycle_stage: String,
    pub parent_project: Option<String>,
    pub status: String,
    pub created_at: String,
}

impl QuestResponse {
    pub fn from_node(node: &Node) -> Self {
        let field_str = |name: &str| -> String {
            match node.fields.get(name) {
                Some(FieldValue::Text(s)) | Some(FieldValue::Enum(s)) => s.clone(),
                Some(FieldValue::RichText(s)) => s.clone(),
                _ => String::new(),
            }
        };
        let field_opt = |name: &str| -> Option<String> {
            match node.fields.get(name) {
                Some(FieldValue::Text(s)) if !s.is_empty() => Some(s.clone()),
                _ => None,
            }
        };

        QuestResponse {
            id: node.id.to_string(),
            title: node.content.clone(),
            description: field_str("description"),
            category: field_str("category"),
            category_custom: field_opt("category_custom"),
            submitter_name: field_opt("submitter_name"),
            lifecycle_stage: {
                let s = field_str("lifecycle_stage");
                if s.is_empty() { "identified".to_string() } else { s }
            },
            parent_project: field_opt("parent_project"),
            status: {
                let s = field_str("status");
                if s.is_empty() { "active".to_string() } else { s }
            },
            created_at: node.created_at.to_rfc3339(),
        }
    }

    /// Returns true if this quest has a contact stored (determines Connect button visibility).
    pub fn has_contact(node: &Node) -> bool {
        matches!(node.fields.get("contact"), Some(FieldValue::Text(s)) if !s.is_empty())
    }
}

/// Form data from the quest submission form.
#[derive(Deserialize)]
pub struct QuestForm {
    pub name: Option<String>,
    pub quest: String,
    pub category: String,
    pub category_custom: Option<String>,
    pub description: String,
    pub contact: Option<String>,
    /// Honeypot field — if filled, it's a bot.
    pub website: Option<String>,
}

#[derive(Deserialize)]
pub struct QuestListQuery {
    pub category: Option<String>,
}

/// POST /api/quests — create a new quest from form submission.
pub async fn create_quest(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Form(form): Form<QuestForm>,
) -> Response {
    // If the user is logged in, associate this quest with their account
    let session_email = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));
    // Honeypot check
    if form.website.as_ref().is_some_and(|w| !w.is_empty()) {
        return Redirect::to("/?submitted=true").into_response();
    }

    let mut fields = BTreeMap::new();
    fields.insert("privacy_tier".to_string(), FieldValue::Enum("public".to_string()));
    fields.insert("status".to_string(), FieldValue::Enum("active".to_string()));
    fields.insert("category".to_string(), FieldValue::Enum(form.category));
    fields.insert(
        "description".to_string(),
        FieldValue::RichText(form.description),
    );
    fields.insert(
        "lifecycle_stage".to_string(),
        FieldValue::Enum("identified".to_string()),
    );

    if let Some(ref custom) = form.category_custom {
        if !custom.is_empty() {
            fields.insert("category_custom".to_string(), FieldValue::Text(custom.clone()));
        }
    }
    if let Some(ref name) = form.name {
        if !name.is_empty() {
            fields.insert("submitter_name".to_string(), FieldValue::Text(name.clone()));
        }
    }
    if let Some(ref contact) = form.contact {
        if !contact.is_empty() {
            fields.insert("contact".to_string(), FieldValue::Text(contact.clone()));
        }
    }
    if let Some(ref email) = session_email {
        fields.insert("submitter_email".to_string(), FieldValue::Text(email.clone()));
    }

    let mut node = Node::new(&form.quest).with_supertag("quest");
    node.fields = fields;

    // Validate against supertag schema
    if let Err(e) = state.registry.validate_node(&node) {
        return (StatusCode::BAD_REQUEST, format!("Validation error: {e}")).into_response();
    }

    // Dual-write: log (canonical) then index (memory)
    {
        let mut log = state.log.lock().unwrap();
        if let Err(e) = log.append(Mutation::CreateNode { node: node.clone() }) {
            tracing::error!("Failed to append to log: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Write failed").into_response();
        }
    }
    {
        let mut index = state.index.write().unwrap();
        index.apply_mutation(Mutation::CreateNode { node: node.clone() });
    }

    let response = QuestResponse::from_node(&node);
    let _ = state.quest_tx.send(QuestEvent::Created { quest: response });

    Redirect::to("/?submitted=true").into_response()
}

/// POST /api/quests/json — create a new quest from JSON (for JS fetch).
pub async fn create_quest_json(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Json(form): Json<QuestForm>,
) -> Response {
    // Honeypot check
    if form.website.as_ref().is_some_and(|w| !w.is_empty()) {
        return (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response();
    }
    let session_email = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));

    let mut fields = BTreeMap::new();
    fields.insert("privacy_tier".to_string(), FieldValue::Enum("public".to_string()));
    fields.insert("status".to_string(), FieldValue::Enum("active".to_string()));
    fields.insert("category".to_string(), FieldValue::Enum(form.category));
    fields.insert(
        "description".to_string(),
        FieldValue::RichText(form.description),
    );
    fields.insert(
        "lifecycle_stage".to_string(),
        FieldValue::Enum("identified".to_string()),
    );

    if let Some(ref custom) = form.category_custom {
        if !custom.is_empty() {
            fields.insert("category_custom".to_string(), FieldValue::Text(custom.clone()));
        }
    }
    if let Some(ref name) = form.name {
        if !name.is_empty() {
            fields.insert("submitter_name".to_string(), FieldValue::Text(name.clone()));
        }
    }
    if let Some(ref contact) = form.contact {
        if !contact.is_empty() {
            fields.insert("contact".to_string(), FieldValue::Text(contact.clone()));
        }
    }
    if let Some(ref email) = session_email {
        fields.insert("submitter_email".to_string(), FieldValue::Text(email.clone()));
    }

    let mut node = Node::new(&form.quest).with_supertag("quest");
    node.fields = fields;

    if let Err(e) = state.registry.validate_node(&node) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))).into_response();
    }

    // Dual-write
    {
        let mut log = state.log.lock().unwrap();
        if let Err(e) = log.append(Mutation::CreateNode { node: node.clone() }) {
            tracing::error!("Failed to append to log: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "write failed"}))).into_response();
        }
    }
    {
        let mut index = state.index.write().unwrap();
        index.apply_mutation(Mutation::CreateNode { node: node.clone() });
    }

    let response = QuestResponse::from_node(&node);
    let _ = state.quest_tx.send(QuestEvent::Created { quest: response.clone() });

    (StatusCode::CREATED, Json(response)).into_response()
}

/// GET /api/quests — list all quests, optionally filtered by category.
pub async fn list_quests(
    State(state): State<QuestHubState>,
    Query(params): Query<QuestListQuery>,
) -> Json<Vec<QuestResponse>> {
    let index = state.index.read().unwrap();
    let quests = index.list_by_supertag("quest");
    let mut responses: Vec<QuestResponse> = quests
        .into_iter()
        .map(QuestResponse::from_node)
        .collect();

    if let Some(ref cat) = params.category {
        responses.retain(|q| q.category == *cat);
    }

    // Newest first
    responses.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Json(responses)
}

/// GET /api/quests/:id — get a single quest by ID.
pub async fn get_quest(
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
) -> Response {
    let node_id = match id.parse::<NodeId>() {
        Ok(nid) => nid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid quest ID").into_response(),
    };

    let index = state.index.read().unwrap();
    match index.get_node(node_id) {
        Some(node) if node.supertag.as_deref() == Some("quest") => {
            Json(QuestResponse::from_node(node)).into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Quest not found").into_response(),
    }
}
