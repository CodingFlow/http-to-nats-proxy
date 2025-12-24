use axum::extract::{Path, Query};
use axum::http::{HeaderMap, Method, StatusCode};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

use crate::AppState;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NatsRequest {
    origin_reply_to: String,
    headers: BTreeMap<String, String>,
    query_parameters: HashMap<String, String>,
    body: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsReponse {
    headers: BTreeMap<String, String>,
    body: Value,
    status_code: u16,
}

pub async fn handler(
    axum::extract::State(shared_state): axum::extract::State<AppState>,
    method: Method,
    path: Path<String>,
    Query(query_parameters): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, String) {
    let client = shared_state.client;
    let subject = create_subject(method, path);

    println!("subject: {subject}");

    let inbox = client.new_inbox();

    let payload = NatsRequest {
        origin_reply_to: inbox.clone(),
        headers: headers
            .iter()
            .map(|(header_name, header_value)| {
                (
                    header_name.to_string(),
                    header_value.to_str().unwrap().to_string(),
                )
            })
            .collect(),
        query_parameters,
        body: match body.is_empty() {
            true => Value::String("".to_string()),
            false => serde_json::from_str::<Value>(&body).unwrap(),
        },
    };

    let mut nats_headers = async_nats::HeaderMap::new();
    let unique_id = nuid::next();
    nats_headers.append(async_nats::header::NATS_MESSAGE_ID, unique_id.as_str());

    let bytes = serde_json::to_vec(&json!(payload)).unwrap();

    let mut subscription = client.subscribe(inbox.clone()).await.unwrap();

    let _ = client
        .publish_with_reply_and_headers(subject, inbox, nats_headers, bytes.into())
        .await;

    println!("sent request");

    let message = subscription.next().await.unwrap();
    let result: NatsReponse = serde_json::from_slice(&message.payload.slice(..)).unwrap();

    let _ = subscription.unsubscribe().await;

    println!("received response");

    (
        StatusCode::from_u16(result.status_code).unwrap(),
        result.body.to_string(),
    )
}

fn create_subject(method: Method, path: Path<String>) -> String {
    let subject_path = &path.split('/').collect::<Vec<&str>>().join(".");
    let lowercase_method = method.to_string().to_lowercase();
    let subject = format!("{lowercase_method}.{subject_path}");
    subject
}
