use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::{io::Cursor, io::Read};
use zip::ZipArchive;

pub fn router(prefix: &str, archive_bytes: &'static [u8]) -> axum::Router {
    let path = format!("{prefix}/*path");
    let handler = move |path: Path<String>| serve_zip(archive_bytes, path);
    Router::new().route(&path, get(handler))
}

async fn serve_zip(archive_bytes: &'static [u8], Path(path): Path<String>) -> impl IntoResponse {
    let cursor = Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(cursor).unwrap();

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
