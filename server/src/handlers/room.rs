use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
};
use chrono::Utc;
use kidur_core::{FieldValue, NodeId};

use crate::handlers::comment::comment_from_node;
use crate::handlers::quest::QuestResponse;
use crate::session_store::get_session_cookie;
use crate::state::QuestHubState;

// ─────────────────────────────────────────────────────────────────────────────
// Auth helper
// ─────────────────────────────────────────────────────────────────────────────

fn auth_email(headers: &axum::http::HeaderMap, state: &QuestHubState) -> Option<String> {
    let tok = get_session_cookie(headers)?;
    state.session_store.get_session_email(&tok)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /my — personal room
// ─────────────────────────────────────────────────────────────────────────────

pub async fn my_room(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Response {
    let email = match auth_email(&headers, &state) {
        Some(e) => e,
        None    => return axum::response::Redirect::to("/login").into_response(),
    };

    let (my_quests, saved_quests) = {
        let index = state.index.read().unwrap();

        let my_quests: Vec<QuestResponse> = index
            .all_quest_nodes()
            .filter(|n| {
                matches!(n.fields.get("submitter_email"),
                    Some(FieldValue::Text(e)) if e.eq_ignore_ascii_case(&email))
            })
            .map(|n| QuestResponse::from_node(n))
            .collect();

        let saved_ids = state.user_store.saved_quest_ids(&email);
        let saved_quests: Vec<QuestResponse> = saved_ids
            .iter()
            .filter_map(|id| {
                let nid: NodeId = id.parse().ok()?;
                index.get_node(nid).map(|n| QuestResponse::from_node(n))
            })
            .collect();

        (my_quests, saved_quests)
    };

    let saved_ids = state.user_store.saved_quest_ids(&email);
    let html = render_room(&email, &my_quests, &saved_quests, &saved_ids);
    Html(html).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /my/save/:id and /my/unsave/:id
// ─────────────────────────────────────────────────────────────────────────────

pub async fn save_quest(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
) -> Response {
    let email = match auth_email(&headers, &state) {
        Some(e) => e,
        None    => return (StatusCode::UNAUTHORIZED, "Please sign in").into_response(),
    };
    state.user_store.save_quest(&email, &id);
    Json(serde_json::json!({"ok": true, "saved": true})).into_response()
}

pub async fn unsave_quest(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
) -> Response {
    let email = match auth_email(&headers, &state) {
        Some(e) => e,
        None    => return (StatusCode::UNAUTHORIZED, "Please sign in").into_response(),
    };
    state.user_store.unsave_quest(&email, &id);
    Json(serde_json::json!({"ok": true, "saved": false})).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /my/export.json and /my/export.md
// ─────────────────────────────────────────────────────────────────────────────

pub async fn export_json(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Response {
    let email = match auth_email(&headers, &state) {
        Some(e) => e,
        None    => return (StatusCode::UNAUTHORIZED, "Please sign in").into_response(),
    };
    let (my_quests, saved_quests, my_comments) = collect_user_data(&email, &state);
    let export = serde_json::json!({
        "exported_at":  Utc::now().to_rfc3339(),
        "user":         { "email": &email },
        "my_quests":    my_quests,
        "saved_quests": saved_quests,
        "my_comments":  my_comments,
    });
    let body = serde_json::to_string_pretty(&export).unwrap_or_default();
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE,        "application/json"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"questhub-room.json\""),
        ],
        body,
    ).into_response()
}

pub async fn export_markdown(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Response {
    let email = match auth_email(&headers, &state) {
        Some(e) => e,
        None    => return (StatusCode::UNAUTHORIZED, "Please sign in").into_response(),
    };
    let (my_quests, saved_quests, my_comments) = collect_user_data(&email, &state);
    let md = build_markdown(&email, &my_quests, &saved_quests, &my_comments);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE,        "text/markdown; charset=utf-8"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"questhub-room.md\""),
        ],
        md,
    ).into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// Data collection (shared between export endpoints)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct CommentExport {
    quest_id:    String,
    quest_title: String,
    body:        String,
    created_at:  String,
}

fn collect_user_data(
    email: &str,
    state: &QuestHubState,
) -> (Vec<QuestResponse>, Vec<QuestResponse>, Vec<CommentExport>) {
    let index = state.index.read().unwrap();

    // Collect quest ids + titles into owned data first, to avoid double-borrow in comment walk
    let all_quest_ids: Vec<(NodeId, String)> = index
        .all_quest_nodes()
        .map(|n| (n.id, n.content.clone()))
        .collect();

    let mut my_quests: Vec<QuestResponse> = index
        .all_quest_nodes()
        .filter(|n| {
            matches!(n.fields.get("submitter_email"),
                Some(FieldValue::Text(e)) if e.eq_ignore_ascii_case(email))
        })
        .map(|n| QuestResponse::from_node(n))
        .collect();
    my_quests.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Walk children of every quest to find comments by this user
    let mut my_comments: Vec<CommentExport> = all_quest_ids
        .iter()
        .flat_map(|(qid, title)| {
            index
                .get_children(*qid)
                .into_iter()
                .filter(|c| {
                    matches!(c.fields.get("commenter_email"),
                        Some(FieldValue::Text(e)) if e.eq_ignore_ascii_case(email))
                })
                .filter_map(|c| {
                    let comment = comment_from_node(c)?;
                    Some(CommentExport {
                        quest_id:    comment.quest_id,
                        quest_title: title.clone(),
                        body:        comment.body,
                        created_at:  comment.created_at,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();
    my_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let saved_ids = state.user_store.saved_quest_ids(email);
    let mut saved_quests: Vec<QuestResponse> = saved_ids
        .iter()
        .filter_map(|id| {
            let nid: NodeId = id.parse().ok()?;
            index.get_node(nid).map(|n| QuestResponse::from_node(n))
        })
        .collect();
    saved_quests.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    (my_quests, saved_quests, my_comments)
}

// ─────────────────────────────────────────────────────────────────────────────
// Markdown builder
// ─────────────────────────────────────────────────────────────────────────────

fn build_markdown(
    email:        &str,
    my_quests:    &[QuestResponse],
    saved_quests: &[QuestResponse],
    my_comments:  &[CommentExport],
) -> String {
    let mut md = format!(
        "# My QuestHub Room\n_Exported: {}_\n_Account: {}_\n\n---\n\n",
        &Utc::now().to_rfc3339()[..10],
        email,
    );

    md.push_str(&format!("## My Quests ({})\n\n", my_quests.len()));
    if my_quests.is_empty() {
        md.push_str("_No quests submitted yet._\n\n");
    } else {
        for q in my_quests {
            md.push_str(&format!(
                "### {}\n**Category:** {} · **Stage:** {} · **Submitted:** {}\n\n{}\n\n[View on QuestHub](https://questhub.eco/quest/{})\n\n---\n\n",
                q.title, q.category, q.lifecycle_stage, &q.created_at[..10],
                q.description, q.id,
            ));
        }
    }

    md.push_str(&format!("## Saved Quests ({})\n\n", saved_quests.len()));
    if saved_quests.is_empty() {
        md.push_str("_No saved quests yet._\n\n");
    } else {
        for q in saved_quests {
            md.push_str(&format!(
                "### {}\n**Category:** {} · **Stage:** {}\n\n{}\n\n[View on QuestHub](https://questhub.eco/quest/{})\n\n---\n\n",
                q.title, q.category, q.lifecycle_stage,
                q.description, q.id,
            ));
        }
    }

    md.push_str(&format!("## My Comments ({})\n\n", my_comments.len()));
    if my_comments.is_empty() {
        md.push_str("_No comments yet._\n\n");
    } else {
        for c in my_comments {
            md.push_str(&format!(
                "**{}** — on [{}](https://questhub.eco/quest/{})\n> {}\n\n",
                &c.created_at[..10], c.quest_title, c.quest_id, c.body,
            ));
        }
    }

    md
}

// ─────────────────────────────────────────────────────────────────────────────
// Room HTML renderer
// ─────────────────────────────────────────────────────────────────────────────

fn render_room(
    email:        &str,
    my_quests:    &[QuestResponse],
    saved_quests: &[QuestResponse],
    saved_ids:    &[String],
) -> String {
    let tmpl = include_str!("../../templates/room.html");

    let my_html = if my_quests.is_empty() {
        r#"<p class="qh-empty-state">You haven't submitted any quests yet.
           <a href="/#submit">Plant your first seed →</a></p>"#.to_string()
    } else {
        my_quests.iter().map(|q| quest_card_html(q, saved_ids)).collect::<Vec<_>>().join("")
    };

    let saved_html = if saved_quests.is_empty() {
        r#"<p class="qh-empty-state">No saved quests yet.
           Browse the <a href="/">garden</a> and save ones that resonate.</p>"#.to_string()
    } else {
        saved_quests.iter().map(|q| quest_card_html(q, saved_ids)).collect::<Vec<_>>().join("")
    };

    tmpl
        .replace("{{USER_EMAIL}}", &esc(email))
        .replace("{{MY_QUESTS_COUNT}}", &my_quests.len().to_string())
        .replace("{{SAVED_QUESTS_COUNT}}", &saved_quests.len().to_string())
        .replace("{{MY_QUESTS_HTML}}", &my_html)
        .replace("{{SAVED_QUESTS_HTML}}", &saved_html)
}

fn quest_card_html(q: &QuestResponse, saved_ids: &[String]) -> String {
    let saved      = saved_ids.contains(&q.id);
    let save_label = if saved { "Saved ✓" } else { "+ Save" };
    let save_class = if saved { "qh-save-btn qh-save-btn--saved" } else { "qh-save-btn" };
    format!(
        r#"<div class="qh-room-card">
  <div class="qh-room-card-top">
    <a href="/quest/{id}" class="qh-room-card-title">{title}</a>
    <button class="{save_class}" data-quest-id="{id}">{save_label}</button>
  </div>
  <p class="qh-room-card-meta">{cat} · {stage} · {date}</p>
  <p class="qh-room-card-desc">{desc}</p>
</div>"#,
        id         = esc(&q.id),
        title      = esc(&q.title),
        cat        = esc(&q.category),
        stage      = esc(&q.lifecycle_stage),
        date       = &q.created_at[..10],
        desc       = esc(&q.description.chars().take(180).collect::<String>()),
        save_class = save_class,
        save_label = save_label,
    )
}
