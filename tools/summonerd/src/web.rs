use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::storage::Storage;
use crate::PhaseMarker;

/// Represents the storage used by the web application.
pub struct WebAppState {
    storage: Storage,
}

pub fn web_app(storage: Storage) -> Router {
    let shared_state = Arc::new(WebAppState { storage });

    Router::new()
        .route("/", get(main_page).with_state(shared_state.clone()))
        .route("/phase/1", get(phase_1).with_state(shared_state.clone()))
        .route("/phase/2", get(phase_2).with_state(shared_state))
}

pub async fn main_page(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    // TODO: Grab from the database, so we will need the state
    let phase_number = 1;
    let template = MainTemplate { phase_number };
    HtmlTemplate(template)
}

pub async fn phase_1(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
    // TODO: Also get info from queue

    let num_contributions_so_far_phase_1 = state
        .storage
        .current_slot(PhaseMarker::P1)
        .await
        .expect("Can get contributions so far");

    let recent_contributions_phase_1 = state
        .storage
        .last_n_contributors(PhaseMarker::P1, 5)
        .await
        .expect("Can get top N contributors");

    let template = Phase1Template {
        num_contributions_so_far_phase_1,
        recent_contributions_phase_1,
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

    let recent_contributions_phase_2 = state
        .storage
        .last_n_contributors(PhaseMarker::P2, 5)
        .await
        .expect("Can get top N contributors");

    let template = Phase2Template {
        num_contributions_so_far_phase_2,
        recent_contributions_phase_2,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "main.html")]
struct MainTemplate {
    phase_number: u64,
}

#[derive(Template)]
#[template(path = "phase1.html")]
struct Phase1Template {
    num_contributions_so_far_phase_1: u64,
    recent_contributions_phase_1: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "phase2.html")]
struct Phase2Template {
    num_contributions_so_far_phase_2: u64,
    recent_contributions_phase_2: Vec<(String, String)>,
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
