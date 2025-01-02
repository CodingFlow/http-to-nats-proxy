use axum::extract::Path;
use axum::http::{HeaderMap, Method, StatusCode};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NatsRequest {
    origin_reply_to: String,
    headers: BTreeMap<String, String>,
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
    method: Method,
    path: Path<String>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, String) {
    let host = env::var("NATS_SERVICE_HOST").unwrap();
    let port = env::var("NATS_SERVICE_PORT").unwrap();
    let nats_url = format!("nats://{host}:{port}");

    let client = async_nats::connect(nats_url).await.unwrap();

    println!("connected to nats");

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
        body: serde_json::from_str::<Value>(&body).unwrap(),
    };

    let bytes = serde_json::to_vec(&json!(payload)).unwrap();

    let mut subscription = client.subscribe(inbox.clone()).await.unwrap();

    let _ = client
        .publish_with_reply(subject, inbox, bytes.into())
        .await;

    let message = subscription.next().await.unwrap();
    let result: NatsReponse = serde_json::from_slice(&message.payload.slice(..)).unwrap();

    let _ = subscription.unsubscribe().await;

    println!("sent request");

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
