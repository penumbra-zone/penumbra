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
const LAST_N: u64 = 50_000;

/// Represents the storage used by the web application.
pub struct WebAppState {
    address: Address,
    config: Config,
    phase: PhaseMarker,
    queue: ParticipantQueue,
    storage: Storage,
}

async fn serve_summoning_jpg() -> impl IntoResponse {
    let jpg = include_bytes!("../templates/static/summoning.jpg").as_slice();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/jpeg")
        .header("Cache-Control", "public, max-age=3600") // Cache for 1 hour
        .body(axum::body::Full::from(jpg))
        .unwrap()
}

async fn serve_css() -> impl IntoResponse {
    let css = include_bytes!("../templates/static/index.css").as_slice();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .header("Cache-Control", "public, max-age=3600") // Cache for 1 hour
        .body(axum::body::Full::from(css))
        .unwrap()
}

async fn serve_woff2(filename: &str) -> impl IntoResponse {
    let data = match filename {
        "Iosevka-Term" => include_bytes!("../templates/static/Iosevka-Term.woff2").as_slice(),
        "PublicSans-Bold" => include_bytes!("../templates/static/PublicSans-Bold.woff2").as_slice(),
        "PublicSans-Regular" => {
            include_bytes!("../templates/static/PublicSans-Regular.woff2").as_slice()
        }
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".into())
                .unwrap()
        }
    };
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "font/woff2")
        .header("Cache-Control", "public, max-age=3600") // Cache for 1 hour
        .body(axum::body::Full::from(data))
        .unwrap()
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
        .route("/static/index.css", get(serve_css))
        .route(
            "/static/Iosevka-Term.woff2",
            get(|| serve_woff2("Iosevka-Term")),
        )
        .route(
            "/static/PublicSans-Bold.woff2",
            get(|| serve_woff2("PublicSans-Bold")),
        )
        .route(
            "/static/PublicSans-Regular.woff2",
            get(|| serve_woff2("PublicSans-Regular")),
        )
        .route("/static/summoning.jpg", get(|| serve_summoning_jpg()))
}

pub async fn main_page(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    let participants_top_median = snapshot_participants_top_median(state.clone()).await;

    let (phase_number, phase_1_participants_top_median, phase_2_participants_top_median) =
        match state.phase {
            PhaseMarker::P1 => (1, Some(participants_top_median), None),
            PhaseMarker::P2 => (2, None, Some(participants_top_median)),
        };

    let phase_1_completed = state
        .storage
        .current_slot(PhaseMarker::P1)
        .await
        .unwrap_or(0);
    let phase_2_completed = state
        .storage
        .current_slot(PhaseMarker::P2)
        .await
        .unwrap_or(0);

    let template = MainTemplate {
        address: state.address.to_string(),
        min_bid: format!("{}penumbra", state.config.min_bid_u64),
        phase_number,
        phase_1_completed,
        phase_2_completed,
        phase_1_participants_top_median,
        phase_2_participants_top_median,
    };
    HtmlTemplate(template)
}

pub async fn snapshot_participants_top_median(state: Arc<WebAppState>) -> (u64, String, String) {
    let snapshot = state.queue.snapshot().await;
    (
        snapshot.connected_participants,
        format!(
            "{}penumbra",
            snapshot.top_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
        ),
        format!(
            "{}penumbra",
            snapshot.median_bid.unwrap_or(0u64.into()) / 1_000_000u128.into()
        ),
    )
}

pub async fn phase_1(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    let contributions_by_slot_hash_time_shortaddr = state
        .storage
        .last_n_contributors(PhaseMarker::P1, LAST_N)
        .await
        .expect("Can get top N contributors");

    let snapshot_participants_top_median = if state.phase == PhaseMarker::P1 {
        Some(snapshot_participants_top_median(state.clone()).await)
    } else {
        None
    };

    // extract the contribution number from the contribution data
    let completed = contributions_by_slot_hash_time_shortaddr
        .first()
        .map(|(n, _, _, _)| *n)
        .unwrap_or(0);

    let template = Phase1Template {
        completed,
        snapshot_participants_top_median,
        contributions_by_slot_hash_time_shortaddr,
    };
    HtmlTemplate(template)
}

pub async fn phase_2(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    let contributions_by_slot_hash_time_shortaddr = state
        .storage
        .last_n_contributors(PhaseMarker::P2, LAST_N)
        .await
        .expect("Can get top N contributors");

    let snapshot_participants_top_median = if state.phase == PhaseMarker::P2 {
        Some(snapshot_participants_top_median(state.clone()).await)
    } else {
        None
    };

    // extract the contribution number from the contribution data
    let completed = contributions_by_slot_hash_time_shortaddr
        .first()
        .map(|(n, _, _, _)| *n)
        .unwrap_or(0);

    let template = Phase2Template {
        completed,
        snapshot_participants_top_median,
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
    phase_1_completed: u64,
    phase_1_participants_top_median: Option<(u64, String, String)>,
    phase_2_completed: u64,
    phase_2_participants_top_median: Option<(u64, String, String)>,
}

#[derive(Template)]
#[template(path = "phase1.html")]
struct Phase1Template {
    completed: u64,
    snapshot_participants_top_median: Option<(u64, String, String)>,
    contributions_by_slot_hash_time_shortaddr: Vec<(u64, String, String, String)>,
}

#[derive(Template)]
#[template(path = "phase2.html")]
struct Phase2Template {
    completed: u64,
    snapshot_participants_top_median: Option<(u64, String, String)>,
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
