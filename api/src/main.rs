//! corral-api: the link shortener's HTTP API.
//!
//! POST /links        {"url": "https://..."} -> the new link
//! GET  /links        every link, most followed first
//! GET  /{slug}       a redirect to the link's URL, counted as a hit
//! GET  /healthz      "ok"
//!
//! Links are kept in memory: this is a demo application.

mod store;

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};

use store::Store;

#[derive(Deserialize)]
struct NewLink {
    url: String,
}

#[derive(Serialize)]
struct Problem {
    error: &'static str,
}

async fn create(State(store): State<Arc<Store>>, Json(new): Json<NewLink>) -> Response {
    if !(new.url.starts_with("http://") || new.url.starts_with("https://")) {
        let problem = Problem {
            error: "url must start with http:// or https://",
        };
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(problem)).into_response();
    }
    (StatusCode::CREATED, Json(store.add(new.url))).into_response()
}

async fn list(State(store): State<Arc<Store>>) -> Response {
    Json(store.all()).into_response()
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
