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

    Router::new().route("/", get(main_page).with_state(shared_state))
}

pub async fn main_page(State(state): State<Arc<WebAppState>>) -> impl IntoResponse {
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

    let template = MainTemplate {
        num_contributions_so_far_phase_1,
        num_contributions_so_far_phase_2,
        recent_contributions_phase_1,
        recent_contributions_phase_2,
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "main.html")]
struct MainTemplate {
    num_contributions_so_far_phase_1: u64,
    num_contributions_so_far_phase_2: u64,
    recent_contributions_phase_2: Vec<(String, String)>,
    recent_contributions_phase_1: Vec<(String, String)>,
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
