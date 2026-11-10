//! corral-api: the link shortener's HTTP API.
//!
//! POST /links        {"url": "https://..."} -> the new link
//! GET  /links        every link, most followed first; ?match=<regex>
//!                    keeps the links whose URL matches the pattern
//! GET  /{slug}       a redirect to the link's URL, counted as a hit
//! GET  /healthz      "ok"
//!
//! Links are kept in memory: this is a demo application.

mod store;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::json;

use store::Store;

#[derive(Deserialize)]
struct NewLink {
    url: String,
}

async fn create(State(store): State<Arc<Store>>, Json(new): Json<NewLink>) -> Response {
    if !(new.url.starts_with("http://") || new.url.starts_with("https://")) {
        let error = json!({"error": "url must start with http:// or https://"});
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response();
    }
    (StatusCode::CREATED, Json(store.add(new.url))).into_response()
}

#[derive(Deserialize)]
struct Filter {
    r#match: Option<String>,
}

async fn list(State(store): State<Arc<Store>>, Query(filter): Query<Filter>) -> Response {
    let mut links = store.all();
    if let Some(pattern) = filter.r#match {
        // The pattern is the caller's own.
        let pattern = match regex::Regex::new(&pattern) {
            Ok(pattern) => pattern,
            Err(problem) => {
                let error = json!({"error": problem.to_string()});
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(error)).into_response();
            }
        };
        links.retain(|link| pattern.is_match(&link.url));
    }
    Json(links).into_response()
}

async fn follow(State(store): State<Arc<Store>>, Path(slug): Path<String>) -> Response {
    match store.hit(&slug) {
        Some(url) => (StatusCode::FOUND, [(header::LOCATION, url)]).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn app(store: Arc<Store>) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/links", get(list).post(create))
        .route("/{slug}", get(follow))
        .with_state(store)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = std::env::var("CORRAL_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("corral-api {} on http://{addr}", env!("CARGO_PKG_VERSION"));
    axum::serve(listener, app(Arc::new(Store::default()))).await
}
