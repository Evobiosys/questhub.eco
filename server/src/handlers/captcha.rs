use axum::{extract::State, response::Json};
use serde::Serialize;

use crate::state::QuestHubState;

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge:    String,
    pub difficulty:   u32,
    pub issued_at_ms: u64,
}

/// GET /api/captcha/challenge
/// Issues a fresh proof-of-work challenge for the submission form.
pub async fn issue_challenge(State(state): State<QuestHubState>) -> Json<ChallengeResponse> {
    let (challenge, difficulty, issued_at_ms) = state.spam_guard.issue_challenge();
    Json(ChallengeResponse { challenge, difficulty, issued_at_ms })
}
