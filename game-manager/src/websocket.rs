use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use web_auth::{models::User, AuthSession};

use crate::{game_manager::GameManager, messages::ClientMessage};

pub trait ExtractGameManager {
    fn game_manager(&self) -> GameManager;
}

pub async fn websocket_handler<T>(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    Path(game_id): Path<String>,
    State(app_state): State<T>,
) -> impl IntoResponse
where
    T: ExtractGameManager,
{
    let manager = app_state.game_manager();
    if !manager.game_exists(&game_id).await || !manager.game_is_active(&game_id).await {
        return http::StatusCode::BAD_REQUEST.into_response();
    }
    ws.on_upgrade(|socket| websocket(socket, auth_session.user, game_id, manager))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
pub async fn websocket(
    stream: WebSocket,
    user: Option<User>,
    game_id: String,
    game_manager: GameManager,
) {
    log::debug!("Websocket upgraded");
    // By splitting, we can send and receive at the same time.
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

    let game_id = game_id.as_str();

    let sender_clone = Arc::clone(&sender);
    let mut rx = game_manager
        .join_game(game_id, sender_clone)
        .await
        .unwrap_or_else(|_| panic!("Failed to join game ({game_id}) from websocket"));

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

    let mut game_sender = None;
    if game_manager.was_playing(game_id, &user).await {
        let resp = game_manager
            .play_game(game_id, &user, Arc::clone(&sender))
            .await;
        match resp {
            Ok(tx) => {
                game_sender = Some(tx);
            }
            Err(e) => {
                log::error!("Error playing game: {e}")
            }
        }
    } else {
        loop {
            tokio::select! {
                _ = (&mut send_task) => break,
                recvd = receiver.next() => {
                    match recvd {
                        Some(Ok(Message::Text(msg))) => {
                            let client_message = serde_json::from_str::<ClientMessage>(&msg);
                            match &client_message {
                                Ok(ClientMessage::PlayGame) => {
                                    log::debug!("Trying to Play");
                                    let resp = game_manager.play_game(game_id, &user, Arc::clone(&sender)).await;
                                    match resp {
                                        Ok(tx) => {
                                            game_sender = Some(tx);
                                            break;
                                        },
                                        Err(e) => {log::error!("Error playing game: {e}")},
                                    }
                                }
                                _ => log::debug!("Non PlayGame message: {client_message:?}: {msg:?}"),
                            }
                        }
                        _ => break,
                    }
                },
            }
        }
    }

    let game_sender = if let Some(game_sender) = game_sender {
        game_sender
    } else {
        let _ = send_task.await;
        return;
    };

    // Spawn a task that takes messages from the websocket and sends them to the game handler
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if game_sender.send(text).await.is_err() {
                return;
            }
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
