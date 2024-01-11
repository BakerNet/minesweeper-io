#![allow(dead_code)]
use anyhow::{anyhow, bail, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::SplitSink, StreamExt};
use http::StatusCode;
use minesweeper_lib::{
    cell::PlayerCell,
    client::{ClientPlayer, Play},
    game::{Minesweeper, PlayOutcome},
};
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc, Mutex, RwLock},
    time::{sleep, Duration},
};

use crate::{
    app::{minesweeper::client::GameMessage, FrontendUser},
    models::{
        game::{Game, Player, PlayerUser},
        user::User,
    },
};

use super::{app::AppState, users::AuthSession};

pub fn router() -> Router<AppState> {
    Router::<AppState>::new().route("/api/websocket/game/:id", get(websocket_handler))
}

#[derive(Clone, Debug)]
struct PlayerHandle {
    id: i64,
    player_id: usize,
    display_name: String,
    ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Clone, Debug)]
struct ViewerHandle {
    ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Debug)]
enum GameEvent {
    Player(PlayerHandle),
    Viewer(ViewerHandle),
    Start,
}

#[derive(Clone, Debug)]
struct GameHandle {
    to_client: broadcast::Sender<String>,
    from_client: mpsc::Sender<String>,
    game_events: mpsc::Sender<GameEvent>,
    players: Vec<PlayerHandle>,
    max_players: u8,
    owner: i64,
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
        let (ch_tx, ch_rx) = mpsc::channel(100);
        let handle = GameHandle {
            to_client: bc_tx.clone(),
            from_client: mp_tx,
            game_events: ch_tx,
            players: Vec::with_capacity(max_players as usize),
            max_players,
            owner: user.id,
        };
        games.insert(game_id.to_string(), handle);
        let self_clone = self.clone();
        tokio::spawn(async move { handle_game(game, self_clone, bc_tx, mp_rx, ch_rx).await });
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

    pub async fn game_is_active(&self, game_id: &str) -> bool {
        let games = self.games.read().await;
        games.contains_key(game_id)
    }

    pub async fn join_game(
        &self,
        game_id: &str,
        ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) -> Result<broadcast::Receiver<String>> {
        let games = self.games.read().await;
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get(game_id).unwrap();
        handle
            .game_events
            .send(GameEvent::Viewer(ViewerHandle { ws_sender }))
            .await?;
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
        let player_id = handle.players.len();
        if player_id >= handle.max_players as usize {
            bail!("Game already has max players")
        }
        Player::add_player(&self.db, game_id, user, player_id as u8).await?;
        handle.players.push(PlayerHandle {
            id: user.id,
            player_id,
            display_name: FrontendUser::display_name_or_anon(&user.display_name),
            ws_sender: ws_sender.clone(),
        });
        {
            let mut send = ws_sender.lock().await;
            let msg = GameMessage::PlayerId(player_id);
            (send).send(Message::Text(msg.to_json())).await?;
        }
        handle
            .game_events
            .send(GameEvent::Player(PlayerHandle {
                id: user.id,
                player_id,
                display_name: FrontendUser::display_name_or_anon(&user.display_name),
                ws_sender: ws_sender.clone(),
            }))
            .await?;
        Ok(handle.from_client.clone())
    }
    // TODO - reconnect

    pub async fn start_game(&self, game_id: &str, user: &User) -> Result<()> {
        let mut games = self.games.write().await;
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get_mut(game_id).unwrap();
        if handle.owner != user.id {
            bail!("Game attempted to be started by non-owner")
        }
        Game::start_game(&self.db, game_id).await?;
        handle.game_events.send(GameEvent::Start).await?;
        Ok(())
    }

    async fn complete_game(&self, game_id: &str, final_board: Vec<Vec<PlayerCell>>) -> Result<()> {
        let mut games = self.games.write().await;
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        Game::complete_game(&self.db, game_id, final_board).await?;
        games.remove(game_id).unwrap();
        Ok(())
    }
}

async fn handle_game_event(
    event: GameEvent,
    player_handles: &mut [Option<PlayerHandle>],
    minesweeper: &mut Minesweeper,
    broadcast: &mut broadcast::Sender<String>,
    started: &mut bool,
) {
    match event {
        GameEvent::Player(player) => {
            player_handles[player.player_id] = Some(player.clone());
            let player_board = minesweeper.player_board(player.player_id);
            {
                let mut player_sender = player.ws_sender.lock().await;
                let player_msg = serde_json::to_string(&GameMessage::GameState(player_board))
                    .expect("GameMessage GameState should be serializable");
                log::debug!("Sending player_msg {:?}", player_msg);
                let _ = player_sender.send(Message::Text(player_msg)).await;
            }

            let players = player_handles
                .iter()
                .map(|item| {
                    item.as_ref().map(|player| ClientPlayer {
                        player_id: player.player_id,
                        username: player.display_name.clone(),
                        dead: false,
                        score: 0,
                    })
                })
                .collect();
            let players_msg = serde_json::to_string(&GameMessage::PlayersState(players))
                .expect("GameMessage PlayerState should be serializable");
            log::debug!("Sending players_msg {:?}", players_msg);
            let _ = broadcast.send(players_msg);
        }
        GameEvent::Viewer(viewer) => {
            let viewer_board = minesweeper.viewer_board();
            {
                let mut viewer_sender = viewer.ws_sender.lock().await;
                let viewer_msg = serde_json::to_string(&GameMessage::GameState(viewer_board))
                    .expect("GameMessage GameState should be serializable");
                log::debug!("Sending viewer_msg {:?}", viewer_msg);
                let _ = viewer_sender.send(Message::Text(viewer_msg)).await;
            }
        }
        GameEvent::Start => {
            *started = true;
        }
    }
}

#[allow(unused)]
async fn handle_message(
    msg: &str,
    player_handles: &[Option<PlayerHandle>],
    minesweeper: &mut Minesweeper,
    broadcast: &mut broadcast::Sender<String>,
    started: bool,
) {
    if !started {
        return;
    }
    let play = match serde_json::from_str::<Play>(msg) {
        Ok(play) => play,
        Err(_) => return,
    };
    if play.player > player_handles.len() {
        return;
    }
    let player = if let Some(player) = &player_handles[play.player] {
        player
    } else {
        return;
    };
    let outcome = minesweeper.play(play.player, play.action, play.point);
    let res = match outcome {
        Ok(res) => res,
        Err(e) => {
            let err_msg = serde_json::to_string(&GameMessage::Error(format!("{:?}", e)))
                .expect("GameMessage Error should be serializable");
            {
                let mut player_sender = player.ws_sender.lock().await;
                let _ = player_sender.send(Message::Text(err_msg)).await;
            }
            todo!(); // send to player
            return;
        }
    };
    match res {
        PlayOutcome::Flag(flag) => {
            let flag_msg =
                serde_json::to_string(&GameMessage::PlayOutcome(PlayOutcome::Flag(flag)))
                    .expect("GameMessage PlayOutcome Flag should be serializable");
            {
                let mut player_sender = player.ws_sender.lock().await;
                let _ = player_sender.send(Message::Text(flag_msg)).await;
            }
        }
        default => {
            let outcome_msg = serde_json::to_string(&GameMessage::PlayOutcome(default))
                .expect("GameMessage PlayOutcome non-Flag should be serializable");
            let score = minesweeper.player_score(player.player_id).unwrap();
            let dead = minesweeper.player_dead(player.player_id).unwrap();
            let player_state = ClientPlayer {
                player_id: player.player_id,
                username: player.display_name.clone(),
                dead,
                score,
            };
            let player_state_message =
                serde_json::to_string(&GameMessage::PlayerUpdate(player_state))
                    .expect("GameMessage PlayerUpdate should be serializable");
            let _ = broadcast.send(outcome_msg);
            let _ = broadcast.send(player_state_message);
        }
    }
}

async fn handle_game(
    game: Game,
    game_manager: GameManager,
    broadcaster: broadcast::Sender<String>,
    receiver: mpsc::Receiver<String>,
    game_events: mpsc::Receiver<GameEvent>,
) {
    let mut receiver = receiver;
    let mut game_events = game_events;
    let mut broadcaster = broadcaster;
    let timeout = sleep(Duration::from_secs(60 * 60)); // timeout after 1 hour
    tokio::pin!(timeout);

    let mut player_handles = vec![None; game.max_players as usize];
    let mut minesweeper = Minesweeper::init_game(
        game.rows as usize,
        game.cols as usize,
        game.num_mines as usize,
        game.max_players as usize,
    )
    .unwrap();
    let mut started = game.is_started; // should always be false

    loop {
        tokio::select! {
            Some(msg) = receiver.recv() => {
                log::debug!("Message received: {}", msg);
                handle_message(&msg, &player_handles, &mut minesweeper, &mut broadcaster, started).await;
                if minesweeper.is_over() {
                    break;
                }
            },
            Some(event) = game_events.recv() => {
                log::debug!("Game update received: {:?}", event);
                handle_game_event(event, &mut player_handles, &mut minesweeper, &mut broadcaster, &mut started).await;
            }
            () = &mut timeout => {
                log::debug!("Game timeout reached {}", game.game_id);
                break;
            },
        }
    }

    let _ = game_manager
        .complete_game(&game.game_id, minesweeper.viewer_board())
        .await
        .map_err(|e| log::error!("Error completing game: {e}"));
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    auth_session: AuthSession,
    Path(game_id): Path<String>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    if !app_state.game_manager.game_exists(&game_id).await
        || !app_state.game_manager.game_is_active(&game_id).await
    {
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
    log::debug!("Websocket upgraded");
    // By splitting, we can send and receive at the same time.
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

    let sender_clone = sender.clone();
    let mut rx = game_manager
        .join_game(&game_id, sender_clone)
        .await
        .unwrap();

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
                        let resp = game_manager.play_game(&game_id, &user, sender.clone()).await;
                        match resp {
                            Ok(tx) => {game_sender = Some(tx);},
                            Err(e) => {log::error!("Error playing game: {}", e)},
                        }

                    }
                    Some(msg) => {
                        log::debug!("Non Play message: {:?}", msg);
                    },
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
