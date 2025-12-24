use std::env;

use axum::{Router, routing::get};
use tokio::signal;
use tower_http::cors::CorsLayer;

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
        .layer(CorsLayer::permissive());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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
