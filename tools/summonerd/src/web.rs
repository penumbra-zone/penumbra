use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use penumbra_keys::Address;
use std::sync::Arc;

use crate::{config::Config, PhaseMarker};
use crate::{queue::ParticipantQueue, storage::Storage};

/// The number of previous contributions to display
const LAST_N: u64 = 10_000;

/// Represents the storage used by the web application.
pub struct WebAppState {
    address: Address,
    config: Config,
    phase: PhaseMarker,
    queue: ParticipantQueue,
    storage: Storage,
}

pub fn web_app(
    address: Address,
    config: Config,
    phase: PhaseMarker,
    queue: ParticipantQueue,
    storage: Storage,
) -> Router {
    let shared_state = Arc::new(WebAppState {
        address,
        config,
        phase,
        queue,
        storage,
    });

    Router::new()
        .route("/", get(main_page).with_state(shared_state.clone()))
        .route("/phase/1", get(phase_1).with_state(shared_state.clone()))
        .route("/phase/2", get(phase_2).with_state(shared_state))
}

pub async fn main_page(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    let phase_number = match state.phase {
        PhaseMarker::P1 => 1,
        PhaseMarker::P2 => 2,
    };

    let template = MainTemplate {
        address: state.address.to_string(),
        min_bid: format!("{}penumbra", state.config.min_bid_u64),
        phase_number,
    };
    HtmlTemplate(template)
}

pub async fn phase_1(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    // TODO: Also get info from queue

    let num_contributions_so_far_phase_1 = state
        .storage
        .current_slot(PhaseMarker::P1)
        .await
        .expect("Can get contributions so far");

    let contributions_by_slot_hash_time_shortaddr = state
        .storage
        .last_n_contributors(PhaseMarker::P1, LAST_N)
        .await
        .expect("Can get top N contributors");

    let snapshot_participants_top_median = if state.phase == PhaseMarker::P1 {
        let snapshot = state.queue.snapshot().await;
        Some((
            snapshot.connected_participants,
            format!(
                "{}penumbra",
                snapshot.top_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
            ),
            format!(
                "{}penumbra",
                snapshot.median_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
            ),
        ))
    } else {
        None
    };

    let template = Phase1Template {
        snapshot_participants_top_median,
        num_contributions_so_far_phase_1,
        contributions_by_slot_hash_time_shortaddr,
    };
    HtmlTemplate(template)
}

pub async fn phase_2(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    // TODO: Also get info from queue

    let num_contributions_so_far_phase_2 = state
        .storage
        .current_slot(PhaseMarker::P2)
        .await
        .expect("Can get contributions so far");

    let contributions_by_slot_hash_time_shortaddr = state
        .storage
        .last_n_contributors(PhaseMarker::P2, LAST_N)
        .await
        .expect("Can get top N contributors");

    let snapshot_participants_top_median = if state.phase == PhaseMarker::P2 {
        let snapshot = state.queue.snapshot().await;
        Some((
            snapshot.connected_participants,
            format!(
                "{}penumbra",
                snapshot.top_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
            ),
            format!(
                "{}penumbra",
                snapshot.median_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
            ),
        ))
    } else {
        None
    };

    let template = Phase2Template {
        snapshot_participants_top_median,
        num_contributions_so_far_phase_2,
        contributions_by_slot_hash_time_shortaddr,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "main.html")]
struct MainTemplate {
    address: String,
    min_bid: String,
    phase_number: u64,
}

#[derive(Template)]
#[template(path = "phase1.html")]
struct Phase1Template {
    snapshot_participants_top_median: Option<(u64, String, String)>,
    num_contributions_so_far_phase_1: u64,
    contributions_by_slot_hash_time_shortaddr: Vec<(u64, String, String, String)>,
}

#[derive(Template)]
#[template(path = "phase2.html")]
struct Phase2Template {
    snapshot_participants_top_median: Option<(u64, String, String)>,
    num_contributions_so_far_phase_2: u64,
    contributions_by_slot_hash_time_shortaddr: Vec<(u64, String, String, String)>,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
