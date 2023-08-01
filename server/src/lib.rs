pub mod game_manager;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Form, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use game_manager::GameManager;
use nanoid::nanoid;
use serde::Deserialize;
use std::sync::Arc;

// Our shared state
pub struct AppState {
    pub game_manager: GameManager,
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
pub async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Game id gets set in the receive loop, if it's valid.
    let mut game_id = String::new();
    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(game) = message {
            check_game(&state, &mut game_id, &game);

            // If not empty we want to quit the loop else we want to quit function.
            if !game_id.is_empty() {
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("Game not found")))
                    .await;

                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.game_manager.join_game(&game_id).unwrap();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            state.game_manager.handle_message(&game_id, &text);
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

fn check_game(state: &AppState, game_id: &mut String, id: &str) {
    let game = state.game_manager.game_exists(id);
    if game {
        game_id.push_str(id);
    }
}

pub async fn create_game(State(state): State<Arc<AppState>>) -> Result<String, AppError> {
    let id = nanoid!(8);
    state.game_manager.new_game(&id)?;
    Ok(id)
}

#[derive(Deserialize, Debug)]
pub struct PlayForm {
    game_id: String,
    user: String,
}

#[axum::debug_handler]
pub async fn play_game(
    State(state): State<Arc<AppState>>,
    Form(PlayForm { game_id, user }): Form<PlayForm>,
) -> Result<String, AppError> {
    state.game_manager.play_game(&game_id, &user)?;
    Ok(String::from("Success"))
}

// Include utf-8 file at **compile** time.
pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
