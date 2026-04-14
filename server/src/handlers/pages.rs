use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use kidur_core::NodeId;
use serde::Deserialize;

use crate::handlers::comment::{comment_from_node, CommentResponse};
use crate::handlers::quest::QuestResponse;
use crate::session_store::get_session_cookie;
use crate::state::QuestHubState;

#[derive(Deserialize)]
pub struct IndexQuery {
    pub submitted: Option<String>,
}

/// GET / — serve the main quest garden page.
pub async fn index_page(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Query(params): Query<IndexQuery>,
) -> Html<String> {
    let quest_count = state.index.read().unwrap().quest_count();
    let submitted   = params.submitted.as_deref() == Some("true");
    let auth_email  = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));
    let html = render_index(quest_count, submitted, auth_email.as_deref());
    Html(html)
}

/// GET /quest/:id — server-rendered quest detail page (shareable URL).
pub async fn quest_detail(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
    Path(id): Path<String>,
) -> Response {
    let _auth_email = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));
    let node_id = match id.parse::<NodeId>() {
        Ok(nid) => nid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid quest ID").into_response(),
    };

    let index = state.index.read().unwrap();
    match index.get_node(node_id) {
        Some(node) if node.supertag.as_deref() == Some("quest") => {
            let quest = QuestResponse::from_node(node);
            let has_contact = QuestResponse::has_contact(node);
            let comments: Vec<CommentResponse> = index
                .get_children(node_id)
                .into_iter()
                .filter_map(comment_from_node)
                .collect();
            Html(render_quest_detail(&quest, has_contact, &comments)).into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Quest not found").into_response(),
    }
}

fn nav_auth_html(auth_email: Option<&str>) -> String {
    match auth_email {
        Some(email) => format!(
            r#"<a href="/my" class="qh-nav-link">My Room</a>
               <a href="/auth/logout" class="qh-nav-link qh-nav-signin" title="Signed in as {e}">Sign Out</a>"#,
            e = html_escape(email)
        ),
        None => r#"<a href="/login" class="qh-nav-link qh-nav-signin">Sign In</a>
                   <a href="/signup" class="qh-nav-btn">Sign Up</a>"#.to_string(),
    }
}

/// GET /about — static informational page (why quests, how it works, data sovereignty, etc.)
pub async fn about_page(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Html<String> {
    let auth_email = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));
    Html(render_about(auth_email.as_deref()))
}

fn render_about(auth_email: Option<&str>) -> String {
    let template = include_str!("../../templates/about.html");
    template.replace("{{NAV_AUTH_AREA}}", &nav_auth_html(auth_email))
}

/// GET /peak — full roadmap page.
pub async fn peak_page(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Html<String> {
    let auth_email = get_session_cookie(&headers)
        .and_then(|tok| state.session_store.get_session_email(&tok));
    Html(render_peak(auth_email.as_deref()))
}

fn render_peak(auth_email: Option<&str>) -> String {
    let template = include_str!("../../templates/peak.html");
    template.replace("{{NAV_AUTH_AREA}}", &nav_auth_html(auth_email))
}

fn render_index(quest_count: usize, submitted: bool, auth_email: Option<&str>) -> String {
    let template = include_str!("../../templates/index.html");
    template
        .replace("{{QUEST_COUNT}}", &format!("{quest_count}"))
        .replace("{{NAV_AUTH_AREA}}", &nav_auth_html(auth_email))
        .replace(
            "{{SUBMITTED_BANNER}}",
            if submitted {
                r#"<div class="qh-submitted-banner" id="submitted-banner">
                    <span>Seed planted! Your quest has been added to the garden.</span>
                    <button onclick="this.parentElement.remove()">&times;</button>
                </div>"#
            } else {
                ""
            },
        )
}

fn render_quest_detail(
    quest: &QuestResponse,
    has_contact: bool,
    comments: &[CommentResponse],
) -> String {
    let template = include_str!("../../templates/quest_detail.html");

    let comments_html = if comments.is_empty() {
        "<p class=\"qh-no-comments\">No comments yet. Be the first.</p>".to_string()
    } else {
        comments
            .iter()
            .map(|c| {
                format!(
                    r#"<div class="qh-comment">
              <div class="qh-comment-meta">
                <strong>{}</strong> <span class="qh-comment-date">{}</span>
              </div>
              <p>{}</p>
            </div>"#,
                    html_escape(&c.commenter_name),
                    &c.created_at[..10],
                    html_escape(&c.body)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let connect_btn = if has_contact {
        format!(
            r#"<button class="qh-btn" onclick="document.getElementById('connect-modal').style.display='flex'">
            Connect with {}</button>"#,
            html_escape(quest.submitter_name.as_deref().unwrap_or("this quester"))
        )
    } else {
        String::new()
    };

    template
        .replace("{{QUEST_TITLE}}", &html_escape(&quest.title))
        .replace("{{QUEST_DESCRIPTION}}", &html_escape(&quest.description))
        .replace("{{QUEST_CATEGORY}}", &html_escape(&quest.category))
        .replace("{{QUEST_LIFECYCLE}}", &html_escape(&quest.lifecycle_stage))
        .replace("{{QUEST_ID}}", &quest.id)
        .replace(
            "{{QUEST_SUBMITTER}}",
            &html_escape(quest.submitter_name.as_deref().unwrap_or("Anonymous")),
        )
        .replace("{{QUEST_DATE}}", &quest.created_at[..10])
        .replace(
            "{{QUEST_PARENT}}",
            quest
                .parent_project
                .as_ref()
                .map(|p| format!(r#"<span class="qh-card-parent">under {}</span>"#, html_escape(p)))
                .as_deref()
                .unwrap_or(""),
        )
        .replace("{{COMMENTS_HTML}}", &comments_html)
        .replace("{{CONNECT_BUTTON}}", &connect_btn)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
