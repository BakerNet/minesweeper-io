// TODO - rework this functionality
use super::game_manager::GameManager;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Form, State,
    },
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use nanoid::nanoid;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use minesweeper_lib::GameMessage;

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
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

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
                let err_msg =
                    serde_json::to_string(&GameMessage::Error(String::from("Game not found")))
                        .unwrap();
                let _ = sender.lock().await.send(Message::Text(err_msg)).await;

                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let (mut rx, game_state) = state.game_manager.join_game(&game_id).unwrap();
    let game_state_msg = serde_json::to_string(&GameMessage::GameState(game_state)).unwrap();
    let _ = sender
        .lock()
        .await
        .send(Message::Text(game_state_msg))
        .await;
    let players_state = state.game_manager.players_state(&game_id).unwrap();
    let players_state_msg =
        serde_json::to_string(&GameMessage::PlayersState(players_state)).unwrap();
    let _ = sender
        .lock()
        .await
        .send(Message::Text(players_state_msg))
        .await;

    let sender_clone = Arc::clone(&sender);
    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender_clone
                .lock()
                .await
                .send(Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let res = state.game_manager.handle_message(&game_id, &text);
            if let Err(e) = res {
                let err_msg =
                    serde_json::to_string(&GameMessage::Error(format!("{:?}", e))).unwrap();
                let _ = sender.lock().await.send(Message::Text(err_msg)).await;
            } else if let Ok(Some(msg)) = res {
                let _ = sender.lock().await.send(Message::Text(msg)).await;
            }
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
    let user_id = state.game_manager.play_game(&game_id, &user)?;
    Ok(format!("{}", user_id))
}

// Include utf-8 file at **compile** time.
pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
