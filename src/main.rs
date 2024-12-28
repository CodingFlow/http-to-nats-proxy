use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};

mod http_handlers;

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/*key",
        get(http_handlers::handler)
            .post(http_handlers::handler)
            .put(http_handlers::handler)
            .patch(http_handlers::handler)
            .head(http_handlers::handler)
            .delete(http_handlers::handler),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
