use std::collections::{BTreeMap, HashMap};
use std::env;

use axum::body::Bytes;
use axum::extract::{Path, Request};
use axum::http::{HeaderMap, Method, StatusCode};

struct NatsRequest {
    headers: BTreeMap<String, String>,
}

pub async fn handler(
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Bytes) {
    let host = env::var("NATS_SERVICE_HOST").unwrap();
    let port = env::var("NATS_SERVICE_PORT").unwrap();
    let nats_url = format!("nats://{host}:{port}");

    let client = async_nats::connect(nats_url).await.unwrap();

    let subject_path = &path.split('/').collect::<Vec<&str>>().join(".");
    let subject = format!("{method}.{subject_path}");

    let response = client.request(subject, "".into());

    (StatusCode::CREATED, body)
}
