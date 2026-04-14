use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
};
use chrono::Utc;
use kidur_core::{FieldValue, NodeId};
use serde::Deserialize;
use uuid::Uuid;

use crate::connect_store::{ConnectStatus, ConnectionRequest};
use crate::state::QuestHubState;

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub name: String,
    pub email: String,
    pub message: String,
}

/// POST /api/quests/:id/connect — submit a connection request
pub async fn request_connect(
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
    Json(body): Json<ConnectRequest>,
) -> Response {
    if body.name.trim().is_empty() || body.email.trim().is_empty() || body.message.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "name, email, and message are required").into_response();
    }

    let quest_id = match id.parse::<NodeId>() {
        Ok(nid) => nid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid quest ID").into_response(),
    };

    let (quest_title, owner_contact) = {
        let index = state.index.read().unwrap();
        let node = match index.get_node(quest_id) {
            Some(n) if n.supertag.as_deref() == Some("quest") => n,
            _ => return (StatusCode::NOT_FOUND, "Quest not found").into_response(),
        };
        let title = node.content.clone();
        let contact = match node.fields.get("contact") {
            Some(FieldValue::Text(s)) if !s.is_empty() => s.clone(),
            _ => return (StatusCode::BAD_REQUEST, "This quest has no contact on file").into_response(),
        };
        (title, contact)
    };

    let token = Uuid::new_v4().to_string();
    let approve_url = format!("https://questhub.eco/api/connect/approve/{token}");

    let req = ConnectionRequest {
        token: token.clone(),
        quest_id: id.clone(),
        quest_title: quest_title.clone(),
        owner_contact: owner_contact.clone(),
        requester_name: body.name.clone(),
        requester_email: body.email.clone(),
        message: body.message.clone(),
        status: ConnectStatus::Pending,
        created_at: Utc::now().to_rfc3339(),
    };

    if let Err(e) = state.connect_store.insert(req) {
        tracing::error!("Failed to store connection request: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "Storage error").into_response();
    }

    let subject = format!("Someone wants to connect about your quest: {quest_title}");
    let email_body = format!(
        "Hello,\n\n\
        {requester} wants to connect about your quest \"{title}\" on QuestHub.\n\n\
        Their message:\n{message}\n\n\
        Their contact will only be shared with you if you click Approve below.\n\n\
        Approve this introduction:\n{url}\n\n\
        If you don't want to connect, simply ignore this email.\n\n\
        — QuestHub\nhttps://questhub.eco",
        requester = body.name,
        title = quest_title,
        message = body.message,
        url = approve_url,
    );

    if let Some(ref email_config) = state.email_config {
        if let Err(e) = crate::email::send_email(email_config, &owner_contact, &subject, &email_body) {
            tracing::error!("Failed to send connection email: {e}");
        }
    } else {
        tracing::warn!("No SMTP config — connection request stored but email not sent (token: {token})");
    }

    (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response()
}

/// GET /api/connect/approve/:token — one-click approval link from email
pub async fn approve_connect(
    State(state): State<QuestHubState>,
    Path(token): Path<String>,
) -> Response {
    let (req, newly_approved) = match state.connect_store.approve(&token) {
        Some(result) => result,
        None => return (StatusCode::NOT_FOUND, "Invalid approval token").into_response(),
    };

    if !newly_approved {
        return (StatusCode::OK, Html(already_approved_page())).into_response();
    }

    let owner_subject = format!("Introduction approved — {}", req.requester_name);
    let owner_body = format!(
        "You approved the connection request for your quest \"{title}\".\n\n\
        Here is {name}'s contact:\n{email}\n\n\
        They said:\n{msg}\n\n\
        — QuestHub",
        title = req.quest_title,
        name = req.requester_name,
        email = req.requester_email,
        msg = req.message,
    );

    let req_subject = format!("Your connection request was approved — {}", req.quest_title);
    let req_body = format!(
        "Good news! The quest owner approved your connection request for \"{title}\".\n\n\
        They have been given your email address ({email}). Expect to hear from them.\n\n\
        — QuestHub\nhttps://questhub.eco/quest/{id}",
        title = req.quest_title,
        email = req.requester_email,
        id = req.quest_id,
    );

    if let Some(ref email_config) = state.email_config {
        let _ = crate::email::send_email(email_config, &req.owner_contact, &owner_subject, &owner_body);
        let _ = crate::email::send_email(email_config, &req.requester_email, &req_subject, &req_body);
    }

    Html(approved_page(&req.quest_title, &req.requester_name)).into_response()
}

fn approved_page(quest_title: &str, requester: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><title>Connection Approved — QuestHub</title>
<link rel="stylesheet" href="/assets/css/style.css"></head>
<body><div class="qh-page"><main class="qh-quest-detail"><div class="qh-container">
<h1>Connection approved</h1>
<p>You approved the connection request from <strong>{requester}</strong> for your quest
<em>{quest_title}</em>.</p>
<p>Both of you have been sent an intro email.</p>
<a href="/" class="qh-btn qh-btn-secondary">Back to the garden</a>
</div></main></div></body></html>"#,
        requester = requester,
        quest_title = quest_title,
    )
}

fn already_approved_page() -> String {
    r#"<!DOCTYPE html><html><head><title>Already approved — QuestHub</title>
<link rel="stylesheet" href="/assets/css/style.css"></head>
<body><div class="qh-page"><main class="qh-quest-detail"><div class="qh-container">
<h1>Already approved</h1><p>This connection was already approved.</p>
<a href="/" class="qh-btn qh-btn-secondary">Back to the garden</a>
</div></main></div></body></html>"#.to_string()
}
