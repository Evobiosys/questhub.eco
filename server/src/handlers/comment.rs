use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Form,
};
use kidur_core::{FieldValue, Node, NodeId};
use kidur_log::Mutation;
use serde::{Deserialize, Serialize};

use crate::state::QuestHubState;

#[derive(Serialize)]
pub struct CommentResponse {
    pub id: String,
    pub quest_id: String,
    pub commenter_name: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CommentForm {
    pub name: Option<String>,
    pub body: String,
    /// Honeypot
    pub website: Option<String>,
}

pub fn comment_from_node(node: &Node) -> Option<CommentResponse> {
    if node.supertag.as_deref() != Some("comment") {
        return None;
    }
    let parent_id = node.parent_id?.to_string();
    let name = match node.fields.get("commenter_name") {
        Some(FieldValue::Text(s)) if !s.is_empty() => s.clone(),
        _ => "Anonymous".to_string(),
    };
    Some(CommentResponse {
        id: node.id.to_string(),
        quest_id: parent_id,
        commenter_name: name,
        body: node.content.clone(),
        created_at: node.created_at.to_rfc3339(),
    })
}

/// POST /api/quests/:id/comments
pub async fn create_comment(
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
    Form(form): Form<CommentForm>,
) -> Response {
    // Honeypot
    if form.website.as_ref().is_some_and(|w| !w.is_empty()) {
        return (StatusCode::CREATED, Json(serde_json::json!({"ok": true}))).into_response();
    }

    if form.body.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Comment body required").into_response();
    }

    let quest_id = match id.parse::<NodeId>() {
        Ok(nid) => nid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid quest ID").into_response(),
    };

    // Verify quest exists
    {
        let index = state.index.read().unwrap();
        if index.get_node(quest_id).is_none() {
            return (StatusCode::NOT_FOUND, "Quest not found").into_response();
        }
    }

    let mut node = Node::new(form.body.trim());
    node.parent_id = Some(quest_id);
    node.supertag = Some("comment".to_string());
    if let Some(ref name) = form.name {
        if !name.is_empty() {
            node.fields.insert(
                "commenter_name".to_string(),
                FieldValue::Text(name.clone()),
            );
        }
    }

    // Dual-write: log then index
    {
        let mut log = state.log.lock().unwrap();
        if let Err(e) = log.append(Mutation::CreateNode { node: node.clone() }) {
            tracing::error!("Failed to append comment: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Write failed").into_response();
        }
    }
    {
        let mut index = state.index.write().unwrap();
        index.apply_mutation(Mutation::CreateNode { node: node.clone() });
    }

    let resp = comment_from_node(&node).unwrap();
    (StatusCode::CREATED, Json(resp)).into_response()
}

/// GET /api/quests/:id/comments
pub async fn list_comments(
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
) -> Response {
    let quest_id = match id.parse::<NodeId>() {
        Ok(nid) => nid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid quest ID").into_response(),
    };

    let index = state.index.read().unwrap();
    let comments: Vec<CommentResponse> = index
        .get_children(quest_id)
        .into_iter()
        .filter_map(comment_from_node)
        .collect();

    Json(comments).into_response()
}
