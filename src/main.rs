use std::env;

use axum::{Router, routing::get};
use axum_otel::{AxumOtelOnFailure, AxumOtelOnResponse, AxumOtelSpanCreator};
use tokio::signal;
use tower_http::{cors::CorsLayer, request_id::MakeRequestId, trace::TraceLayer};
use tracing::Level;
use tracing_otel_extra::{LogFormat, Logger};
use uuid::Uuid;

mod http_handlers;

#[derive(Clone)]
struct AppState{
    client: async_nats::Client
}

#[tokio::main]
async fn main() {
    let host = env::var("NATS_SERVICE_HOST").unwrap();
    let port = env::var("NATS_SERVICE_PORT").unwrap();
    let nats_url = format!("nats://{host}:{port}");

    let client = async_nats::connect(nats_url).await.unwrap();

    println!("connected to nats");

    let shared_state = AppState {client: client};

    let _guard = Logger::from_env(Some("OTEL_")).unwrap()
        .with_format(LogFormat::Json)
        .init()
        .expect("Failed to initialize tracing");
    
    let app = Router::new()
        .route(
            "/{*key}",
            get(http_handlers::handler)
                .post(http_handlers::handler)
                .put(http_handlers::handler)
                .patch(http_handlers::handler)
                .head(http_handlers::handler)
                .delete(http_handlers::handler),
        )
        .with_state(shared_state)
        .layer(
            tower::ServiceBuilder::new()
                // .layer(SetRequestIdLayer::x_request_id(MakeRequestUuidV7))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(AxumOtelSpanCreator::new().level(Level::INFO))
                        .on_response(AxumOtelOnResponse::new().level(Level::INFO))
                        .on_failure(AxumOtelOnFailure::new().level(Level::ERROR)),
                )
                // .layer(PropagateRequestIdLayer::x_request_id()),
        )
        .layer(CorsLayer::permissive());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

#[derive(Clone, Copy)]
pub struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(&mut self, _request: &axum::http::Request<B>) -> Option<tower_http::request_id::RequestId> {
        let request_id: axum::http::HeaderValue = Uuid::now_v7().to_string().parse().unwrap();
        let str_id = request_id.to_str().unwrap();
        println!("~~ generated request id: {str_id}");
        Some(tower_http::request_id::RequestId::new(request_id))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
