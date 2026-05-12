use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

use crate::handlers::{auth, captcha, comment, connect, pages, quest, room, sse};
use crate::state::QuestHubState;

pub fn build_router(state: QuestHubState, static_dir: &str) -> Router {
    Router::new()
        // ── Auth ──────────────────────────────────────────────────────────────
        .route("/login",              get(auth::login_page))
        .route("/signup",             get(auth::signup_page))
        .route("/auth/send-magic-link", post(auth::send_magic_link))
        .route("/auth/verify/:token", get(auth::verify_magic_link))
        .route("/auth/logout",        get(auth::logout))
        .route("/auth/oidc/infomaniak/start", get(auth::oidc_infomaniak_start))
        .route("/auth/oidc/gitlab/start",     get(auth::oidc_gitlab_start))
        .route("/auth/oidc/callback",         get(auth::oidc_callback))

        // ── Personal room ─────────────────────────────────────────────────────
        .route("/my",                 get(room::my_room))
        .route("/my/save/:id",        post(room::save_quest))
        .route("/my/unsave/:id",      post(room::unsave_quest))
        .route("/my/export.json",     get(room::export_json))
        .route("/my/export.md",       get(room::export_markdown))

        // ── Pages (HTML) ──────────────────────────────────────────────────────
        .route("/",                   get(pages::index_page))
        .route("/about",              get(pages::about_page))
        .route("/peak",               get(pages::peak_page))
        .route("/quest/:id",          get(pages::quest_detail))

        // ── Quest API (JSON) ──────────────────────────────────────────────────
        .route("/api/quests",         get(quest::list_quests).post(quest::create_quest))
        .route("/api/quests/json",    post(quest::create_quest_json))

        // ── Comments — must come before /api/quests/:id ───────────────────────
        .route("/api/quests/:id/comments",
            get(comment::list_comments).post(comment::create_comment))
        .route("/api/quests/:id/connect", post(connect::request_connect))
        .route("/api/quests/:id",     get(quest::get_quest))

        // ── Connect approval ──────────────────────────────────────────────────
        .route("/api/connect/approve/:token", get(connect::approve_connect))

        // ── SSE + Health ──────────────────────────────────────────────────────
        .route("/api/quests/stream",  get(sse::quest_stream))
        .route("/api/captcha/challenge", get(captcha::issue_challenge))
        .route("/health",             get(|| async { "ok" }))

        // ── Static files ──────────────────────────────────────────────────────
        .nest_service("/assets",      ServeDir::new(static_dir))
        .with_state(state)
}
