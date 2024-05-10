use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::{io::Cursor, io::Read};
use tracing::Instrument;
use zip::ZipArchive;

pub fn router(prefix: &str, archive_bytes: &'static [u8]) -> axum::Router {
    // Create a span describing the router to wrap request handling with.
    let span = tracing::error_span!("zipserve", prefix = prefix);
    let span1 = span.clone();
    let span2 = span.clone();
    // Per Axum docs, wildcard captures don't match empty segments, so we need
    // a special route for the route path.
    let path1 = format!("{prefix}");
    assert!(prefix.ends_with("/"), "prefix must end in a /");
    let path2 = format!("{prefix}*path");
    let handler1 =
        move || serve_zip(archive_bytes, Path("index.html".to_string())).instrument(span1);
    let handler2 = move |path: Path<String>| serve_zip(archive_bytes, path).instrument(span2);
    Router::new()
        .route(&path1, get(handler1))
        .route(&path2, get(handler2))
}

#[allow(clippy::unwrap_used)]
async fn serve_zip(
    archive_bytes: &'static [u8],
    Path(mut path): Path<String>,
) -> impl IntoResponse {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();

    // Rewrite paths ending in / to /index.html
    if path.ends_with('/') {
        tracing::debug!(orig_path = ?path, "rewriting to index.html");
        path.push_str("index.html");
    }

    let rsp = if let Ok(mut file) = archive.by_name(&path) {
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).unwrap();

        // Make a best-guess at the mime-type or else use octet-stream
        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        tracing::debug!(path = ?path, mime_type = ?mime_type, len = ?contents.len(), "serving file");

        Response::builder()
            .header("Content-Type", mime_type)
            .body(axum::body::Body::from(contents))
            .unwrap()
    } else {
        tracing::debug!(path = ?path, "file not found in archive");

        Response::builder()
            .status(404)
            .body("File not found in archive".into())
            .unwrap()
    };

    rsp
}
