use std::collections::{BTreeMap, HashMap};
use std::env;

use axum::body::Bytes;
use axum::extract::{Path, Request};
use axum::http::{HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize)]
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

    println!("connected to nats");

    let subject_path = &path.split('/').collect::<Vec<&str>>().join(".");
    let lowercase_method = method.to_string().to_lowercase();
    let subject = format!("{lowercase_method}.{subject_path}");

    println!("subject: {subject}");

    let payload = NatsRequest {
        headers: headers
            .iter()
            .map(|(header_name, header_value)| {
                (
                    header_name.to_string(),
                    header_value.to_str().unwrap().to_string(),
                )
            })
            .collect(),
    };

    let bytes = serde_json::to_vec(&json!(payload)).unwrap();

    let response = client.request(subject, bytes.into()).await;

    println!("sent request");

    (StatusCode::CREATED, body)
}
