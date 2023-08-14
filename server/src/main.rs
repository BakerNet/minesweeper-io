use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use server::{
    create_game, game_manager::GameManager, index, play_game, websocket_handler, AppState,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_chat=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set up application state for use with with_state().

    let app_state = Arc::new(AppState {
        game_manager: GameManager::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST]);
    let app = Router::new()
        .route("/api/new", post(create_game))
        .route("/api/play", post(play_game))
        .route("/api/websocket", get(websocket_handler))
        .route("/public/*path", get(index))
        .route("/", get(index))
        .route("/*path", get(index))
        .layer(cors)
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
