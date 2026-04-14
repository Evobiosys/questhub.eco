use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_core::Stream;
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::state::QuestHubState;

/// GET /api/quests/stream — SSE endpoint for real-time quest updates.
pub async fn quest_stream(
    State(state): State<QuestHubState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.quest_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let data = serde_json::to_string(&event).ok()?;
            Some(Ok(Event::default().event("quest_update").data(data)))
        }
        Err(_) => None,
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
