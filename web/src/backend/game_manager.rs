#![allow(dead_code)]
use anyhow::{anyhow, bail, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::SplitSink, StreamExt};
use http::StatusCode;
use minesweeper::game::Minesweeper;
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

use crate::{
    app::FrontendUser,
    models::{
        game::{Game, Player, PlayerUser},
        user::User,
    },
};

use super::{app::AppState, users::AuthSession};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("api/websocket/game/:id", get(websocket_handler))
}

#[derive(Clone, Debug)]
struct PlayerHandle {
    id: i64,
    display_name: String,
    ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Clone, Debug)]
struct GameHandle {
    to_client: broadcast::Sender<String>,
    from_client: mpsc::Sender<String>,
    players: Vec<PlayerHandle>,
    max_players: u8,
}

#[derive(Clone, Debug)]
pub struct GameManager {
    db: SqlitePool,
    games: Arc<RwLock<HashMap<String, GameHandle>>>,
}

impl GameManager {
    pub fn new(db: SqlitePool) -> Self {
        GameManager {
            db,
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new_game(
        &self,
        user: &User,
        game_id: &str,
        rows: i64,
        cols: i64,
        num_mines: i64,
        max_players: u8,
    ) -> Result<()> {
        let mut games = self.games.write().await;
        if games.contains_key(game_id) {
            bail!("Game with id {game_id} already exists")
        }
        let game =
            Game::create_game(&self.db, game_id, user, rows, cols, num_mines, max_players).await?;
        let (bc_tx, _bc_rx) = broadcast::channel(100);
        let (mp_tx, mp_rx) = mpsc::channel(100);
        let handle = GameHandle {
            to_client: bc_tx.clone(),
            from_client: mp_tx,
            players: Vec::with_capacity(max_players as usize),
            max_players,
        };
        games.insert(game_id.to_string(), handle);
        let db_clone = self.db.clone();
        tokio::spawn(async move { handle_game(game, db_clone, bc_tx, mp_rx) });
        Ok(())
    }

    pub async fn game_exists(&self, game_id: &str) -> bool {
        Game::get_game(&self.db, game_id).await.is_ok()
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Game> {
        Game::get_game(&self.db, game_id)
            .await?
            .ok_or(anyhow!("Game does not exist"))
    }

    pub async fn get_players(&self, game_id: &str) -> Result<Vec<PlayerUser>> {
        Player::get_players(&self.db, game_id)
            .await
            .map_err(|e| e.into())
    }

    pub async fn join_game(&self, game_id: &str) -> Result<broadcast::Receiver<String>> {
        let games = self.games.read().await;
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get(game_id).unwrap();
        Ok(handle.to_client.subscribe())
    }

    pub async fn play_game(
        &self,
        game_id: &str,
        user: &User,
        ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) -> Result<mpsc::Sender<String>> {
        let mut games = self.games.write().await;
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get_mut(game_id).unwrap();
        if handle.players.len() >= handle.max_players as usize {
            bail!("Game already has max players")
        }
        let _player =
            Player::add_player(&self.db, game_id, user, handle.players.len() as u8).await?;
        handle.players.push(PlayerHandle {
            id: user.id,
            display_name: FrontendUser::display_name_or_anon(&user.display_name),
            ws_sender,
        });
        Ok(handle.from_client.clone())
    }
    // TODO - reconnect
}

async fn handle_game(
    game: Game,
    _db: SqlitePool,
    _broadcaster: broadcast::Sender<String>,
    _receiver: mpsc::Receiver<String>,
) -> () {
    let mut _minesweeper = Minesweeper::init_game(
        game.rows as usize,
        game.cols as usize,
        game.num_mines as usize,
        game.max_players as usize,
    )
    .unwrap();

    todo!()
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    Path(game_id): Path<String>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    if !app_state.game_manager.game_exists(&game_id).await {
        return StatusCode::BAD_REQUEST.into_response();
    }
    ws.on_upgrade(|socket| websocket(socket, auth_session.user, game_id, app_state.game_manager))
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
    // By splitting, we can send and receive at the same time.
    let (sender, mut receiver) = stream.split();

    let mut rx = game_manager.join_game(&game_id).await.unwrap();

    let sender = Arc::new(Mutex::new(sender));
    let sender_clone = sender.clone();
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

    let user = if let Some(user) = user {
        user
    } else {
        let _ = send_task.await;
        return;
    };

    let mut game_sender = None;
    loop {
        tokio::select! {
            _ = (&mut send_task) => break,
            recvd = receiver.next() => {
                match recvd {
                    Some(Ok(Message::Text(msg))) if msg == "Play" => {
                        game_sender = game_manager.play_game(&game_id, &user, sender.clone()).await.ok();
                    }
                    _ => break,
                }
            },
        }
    }

    let game_sender = if let Some(game_sender) = game_sender {
        game_sender
    } else {
        let _ = send_task.await;
        return;
    };

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Err(_) = game_sender.send(text).await {
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
