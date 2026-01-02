use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, Method, header};
use axum::response::Response;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{json, Value};
use tracing_otel_extra::opentelemetry::global::get_text_map_propagator;
use tracing_otel_extra::opentelemetry::propagation::Injector;
use tracing_otel_extra::tracing_opentelemetry::OpenTelemetrySpanExt;
use std::collections::{BTreeMap, HashMap};

use crate::AppState;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NatsRequest<'a> {
    origin_reply_to: String,
    headers: BTreeMap<String, String>,
    query_parameters: HashMap<String, String>,
    #[serde(borrow)]
    body: &'a RawValue,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatsReponse<'a> {
    headers: BTreeMap<String, String>,
    #[serde(borrow)]
    body: &'a RawValue,
    status_code: u16,
}

pub async fn handler(
    axum::extract::State(shared_state): axum::extract::State<AppState>,
    method: Method,
    path: Path<String>,
    Query(query_parameters): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let client = shared_state.client;
    let subject = create_subject(method, path);

    tracing::info!(subject);

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
            true => serde_json::from_str("{}").unwrap(),
            false => serde_json::from_slice(&body).unwrap(),
        },
    };

    let mut nats_headers = async_nats::HeaderMap::new();
    let request_id = headers.get("x-request-id").unwrap().to_str().unwrap();
    println!("x-request-id: {request_id}");
    nats_headers.append(async_nats::header::NATS_MESSAGE_ID, request_id);

    let context = tracing::Span::current().context();
    get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut MyNatsInjector(&mut nats_headers));
    });

    let bytes = serde_json::to_vec(&json!(payload)).unwrap();

    let mut subscription = client.subscribe(inbox.clone()).await.unwrap();

    let _ = client
        .publish_with_reply_and_headers(subject, inbox, nats_headers, bytes.into())
        .await;

    tracing::info!("sent request");

    let message = subscription.next().await.unwrap();
    let response_payload = message.payload;
    let result: NatsReponse = serde_json::from_slice(&response_payload).unwrap();

    let _ = subscription.unsubscribe().await;

    tracing::info!(result.status_code, "received response");
    tracing::debug!("body: {0}", result.body.get());

    let result_body_string = result.body.get();
    let http_response_body = if result_body_string == "{}" {
        Body::empty()
    } else {
        Body::from(result_body_string.to_string())
    };
    
    Response::builder()
        .status(result.status_code)
        .header(header::CONTENT_TYPE, "application/json")
        .body(http_response_body)
        .unwrap()
}

fn create_subject(method: Method, path: Path<String>) -> String {
    let subject_path = &path.split('/').collect::<Vec<&str>>().join(".");
    let lowercase_method = method.to_string().to_lowercase();
    let subject = format!("{lowercase_method}.{subject_path}");
    subject
}

struct MyNatsInjector<'a>(&'a mut async_nats::HeaderMap);

impl<'a> Injector for MyNatsInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key, value);
    }
}
