use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    Form,
};
use serde::Deserialize;
use std::env;

use crate::session_store::get_session_cookie;
use crate::state::QuestHubState;

// ── OIDC endpoints ───────────────────────────────────────────────────────────
const IK_AUTH_URL:     &str = "https://login.infomaniak.com/authorize";
const IK_TOKEN_URL:    &str = "https://login.infomaniak.com/token";
const IK_USERINFO_URL: &str = "https://login.infomaniak.com/userinfo";
// GitLab.com — self-hosted can override via GITLAB_HOST env var
const GL_AUTH_PATH:    &str = "/oauth/authorize";
const GL_TOKEN_PATH:   &str = "/oauth/token";
const GL_USERINFO_PATH:&str = "/oauth/userinfo";
const REDIRECT_URI:    &str = "https://questhub.eco/auth/oidc/callback";

#[derive(Deserialize)]
pub struct MagicLinkForm {
    pub email:   String,
    /// Honeypot
    pub website: Option<String>,
}

/// GET /login
pub async fn login_page() -> Html<&'static str> {
    Html(include_str!("../../templates/login.html"))
}

/// GET /signup — sign-up is the same as sign-in (magic link creates account)
pub async fn signup_page() -> Html<&'static str> {
    Html(include_str!("../../templates/login.html"))
}

/// POST /auth/send-magic-link
/// Sends a login email with a one-time link. Returns JSON.
pub async fn send_magic_link(
    State(state): State<QuestHubState>,
    Form(form): Form<MagicLinkForm>,
) -> Response {
    // Honeypot
    if form.website.as_ref().is_some_and(|w| !w.is_empty()) {
        return Json(serde_json::json!({"ok": true})).into_response();
    }

    let email = form.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Please enter a valid email address."})),
        ).into_response();
    }

    let token     = state.session_store.create_magic_link(&email);
    let magic_url = format!("https://questhub.eco/auth/verify/{token}");

    let subject = "Your QuestHub sign-in link".to_string();
    let body    = format!(
        "Hello,\n\n\
        Here is your sign-in link for QuestHub. It expires in 15 minutes.\n\n\
        {magic_url}\n\n\
        If you didn't request this, you can safely ignore this email.\n\n\
        — QuestHub\nhttps://questhub.eco"
    );

    if let Some(ref cfg) = state.email_config {
        if let Err(e) = crate::email::send_email(cfg, &email, &subject, &body) {
            tracing::error!("Magic link email failed to {email}: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Could not send email. Please try again."})),
            ).into_response();
        }
    } else {
        // Dev mode: log the link so it's usable without SMTP
        tracing::warn!("SMTP not configured — magic link: {magic_url}");
    }

    Json(serde_json::json!({
        "ok": true,
        "message": "Check your inbox — we sent you a sign-in link."
    })).into_response()
}

/// GET /auth/verify/:token — verify magic link, set session cookie, redirect to /my
pub async fn verify_magic_link(
    State(state): State<QuestHubState>,
    Path(token): Path<String>,
) -> Response {
    let email = match state.session_store.verify_magic_link(&token) {
        Some(e) => e,
        None    => return (StatusCode::GONE, Html(
            r#"<!DOCTYPE html><html><head><title>Link Expired — QuestHub</title>
            <link rel="stylesheet" href="/assets/css/style.css"></head>
            <body><div class="qh-page"><main><div class="qh-container" style="padding-top:8rem;text-align:center">
            <h2>This sign-in link has expired</h2>
            <p>Magic links are valid for 15 minutes. Please request a new one.</p>
            <a href="/login" class="qh-btn" style="display:inline-block;margin-top:1.5rem">Back to sign in</a>
            </div></main></div></body></html>"#
        )).into_response(),
    };

    // Ensure user profile exists
    state.user_store.get_or_create(&email);

    let session_tok = state.session_store.create_session(&email);
    let cookie      = format!(
        "qh_session={session_tok}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000"
    );

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/my")
        .header(header::SET_COOKIE, &cookie)
        .body(axum::body::Body::empty())
        .unwrap()
}

/// GET /auth/oidc/infomaniak/start — redirect to Infomaniak OAuth2 login
pub async fn oidc_infomaniak_start(
    State(state): State<QuestHubState>,
) -> Response {
    let client_id = match env::var("INFOMANIAK_CLIENT_ID") {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("INFOMANIAK_CLIENT_ID not set — OIDC unavailable");
            return Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header(header::LOCATION, "/login?error=oidc_not_configured")
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    // CSRF state token — sentinel encodes the provider for the callback
    let csrf_state = state.session_store.create_magic_link("__oidc_ik__");

    let auth_url = format!(
        "{IK_AUTH_URL}?response_type=code&client_id={}&redirect_uri={}&scope=openid+email+profile&state={}",
        urlencode(&client_id),
        urlencode(REDIRECT_URI),
        urlencode(&csrf_state),
    );

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, &auth_url)
        .body(axum::body::Body::empty())
        .unwrap()
}

#[derive(Deserialize)]
pub struct OidcCallbackQuery {
    pub code:  Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct UserinfoResponse {
    email: Option<String>,
}

/// GET /auth/oidc/callback — handle Infomaniak OAuth2 code exchange
pub async fn oidc_callback(
    State(state): State<QuestHubState>,
    Query(q): Query<OidcCallbackQuery>,
) -> Response {
    if let Some(ref err) = q.error {
        tracing::warn!("OIDC error from provider: {err}");
        return redirect_login("Sign-in was cancelled or failed.");
    }

    let code = match q.code {
        Some(ref c) if !c.is_empty() => c.clone(),
        _ => return redirect_login("Missing authorization code."),
    };

    // CSRF check — sentinel encodes which provider started the flow
    let provider = match q.state {
        Some(ref st) => match state.session_store.verify_magic_link(st).as_deref() {
            Some("__oidc_ik__") => "infomaniak",
            Some("__oidc_gl__") => "gitlab",
            _ => return redirect_login("Invalid or expired OIDC state. Please try again."),
        },
        None => return redirect_login("Missing OIDC state parameter."),
    };

    // Resolve endpoints + credentials for this provider
    let (token_url, userinfo_url, client_id, client_secret) = match provider {
        "infomaniak" => {
            let id  = match env::var("INFOMANIAK_CLIENT_ID")     { Ok(v) => v, Err(_) => return redirect_login("Infomaniak OIDC not configured.") };
            let sec = match env::var("INFOMANIAK_CLIENT_SECRET") { Ok(v) => v, Err(_) => return redirect_login("Infomaniak OIDC not configured.") };
            (IK_TOKEN_URL.to_string(), IK_USERINFO_URL.to_string(), id, sec)
        }
        "gitlab" => {
            let id  = match env::var("GITLAB_CLIENT_ID")     { Ok(v) => v, Err(_) => return redirect_login("GitLab OIDC not configured.") };
            let sec = match env::var("GITLAB_CLIENT_SECRET") { Ok(v) => v, Err(_) => return redirect_login("GitLab OIDC not configured.") };
            let host = env::var("GITLAB_HOST").unwrap_or_else(|_| "https://gitlab.com".to_string());
            (format!("{host}{GL_TOKEN_PATH}"), format!("{host}{GL_USERINFO_PATH}"), id, sec)
        }
        _ => return redirect_login("Unknown OAuth provider."),
    };

    // Exchange code for token
    let http = reqwest::Client::new();

    let token_resp: TokenResponse = {
        let r = match http
            .post(&token_url)
            .form(&[
                ("grant_type",    "authorization_code"),
                ("code",          code.as_str()),
                ("redirect_uri",  REDIRECT_URI),
                ("client_id",     client_id.as_str()),
                ("client_secret", client_secret.as_str()),
            ])
            .send()
            .await
        {
            Ok(r)  => r,
            Err(e) => { tracing::error!("OIDC token request failed ({provider}): {e}"); return redirect_login("Sign-in failed."); }
        };
        if !r.status().is_success() {
            tracing::error!("OIDC token endpoint returned {} ({provider})", r.status());
            return redirect_login("Sign-in failed.");
        }
        match r.json().await {
            Ok(t)  => t,
            Err(e) => { tracing::error!("OIDC token parse error ({provider}): {e}"); return redirect_login("Sign-in failed."); }
        }
    };

    // Fetch user email from userinfo endpoint
    let userinfo: UserinfoResponse = {
        let r = match http
            .get(&userinfo_url)
            .bearer_auth(&token_resp.access_token)
            .send()
            .await
        {
            Ok(r)  => r,
            Err(e) => { tracing::error!("OIDC userinfo request failed ({provider}): {e}"); return redirect_login("Sign-in failed."); }
        };
        if !r.status().is_success() {
            tracing::error!("OIDC userinfo returned {} ({provider})", r.status());
            return redirect_login("Sign-in failed.");
        }
        match r.json().await {
            Ok(u)  => u,
            Err(e) => { tracing::error!("OIDC userinfo parse error ({provider}): {e}"); return redirect_login("Sign-in failed."); }
        }
    };

    let email = match userinfo.email {
        Some(ref e) if !e.is_empty() => e.clone(),
        _ => return redirect_login("No email returned by OAuth provider."),
    };

    // Create session
    state.user_store.get_or_create(&email);
    let session_tok = state.session_store.create_session(&email);
    let cookie = format!("qh_session={session_tok}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000");

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/my")
        .header(header::SET_COOKIE, &cookie)
        .body(axum::body::Body::empty())
        .unwrap()
}

/// GET /auth/oidc/gitlab/start — redirect to GitLab OAuth2 login
/// Supports self-hosted GitLab via GITLAB_HOST env var (default: gitlab.com)
pub async fn oidc_gitlab_start(
    State(state): State<QuestHubState>,
) -> Response {
    let client_id = match env::var("GITLAB_CLIENT_ID") {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("GITLAB_CLIENT_ID not set — GitLab OIDC unavailable");
            return Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header(header::LOCATION, "/login?error=gitlab_not_configured")
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };
    let host = env::var("GITLAB_HOST").unwrap_or_else(|_| "https://gitlab.com".to_string());
    let csrf_state = state.session_store.create_magic_link("__oidc_gl__");

    let auth_url = format!(
        "{host}{GL_AUTH_PATH}?client_id={}&redirect_uri={}&response_type=code&scope=openid+email&state={}",
        urlencode(&client_id),
        urlencode(REDIRECT_URI),
        urlencode(&csrf_state),
    );

    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, &auth_url)
        .body(axum::body::Body::empty())
        .unwrap()
}

fn redirect_login(msg: &str) -> Response {
    let encoded = urlencode(msg);
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, format!("/login?error={encoded}"))
        .body(axum::body::Body::empty())
        .unwrap()
}

fn urlencode(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        _ => format!("%{:02X}", c as u32),
    }).collect()
}

/// GET /auth/logout — clear session cookie and redirect home
pub async fn logout(
    headers: axum::http::HeaderMap,
    State(state): State<QuestHubState>,
) -> Response {
    if let Some(tok) = get_session_cookie(&headers) {
        state.session_store.invalidate(&tok);
    }
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .header(header::SET_COOKIE, "qh_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
        .body(axum::body::Body::empty())
        .unwrap()
}
