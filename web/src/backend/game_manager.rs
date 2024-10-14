use ::chrono::{DateTime, Utc};
use anyhow::{anyhow, bail, Result};
use axum::extract::ws::{Message, WebSocket};
use chrono::TimeDelta;
use futures::{sink::SinkExt, stream::SplitSink};
use minesweeper_lib::{
    board::Board,
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Minesweeper, MinesweeperBuilder, MinesweeperOpts, Play, PlayOutcome},
};
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{broadcast, mpsc, Mutex, RwLock},
    time::{interval, Duration},
};

use crate::{
    app::FrontendUser,
    messages::{ClientMessage, GameMessage},
    models::{
        game::{
            Game, GameLog, GameParameters, Player, PlayerGame, PlayerUser, SimpleGameWithPlayers,
        },
        user::User,
    },
};

use super::cache::CachedValue;

#[derive(Clone, Debug)]
struct PlayerHandle {
    user_id: Option<i64>,
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
    owner: Option<i64>,
    start_time: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct GameManager {
    db: SqlitePool,
    games: Arc<RwLock<HashMap<String, GameHandle>>>,
    // use active cache to avoid frequent read locks on games
    active_cache: Arc<CachedValue<Vec<SimpleGameWithPlayers>>>,
    recent_cache: Arc<CachedValue<Vec<SimpleGameWithPlayers>>>,
}

impl GameManager {
    pub fn new(db: SqlitePool) -> Self {
        GameManager {
            db,
            games: RwLock::new(HashMap::new()).into(),
            // 1.5 second active cache
            active_cache: CachedValue::new(Duration::from_millis(1500)).into(),
            recent_cache: CachedValue::new(Duration::from_secs(4)).into(),
        }
    }

    pub async fn new_game(
        &self,
        user: Option<User>,
        game_id: &str,
        game_parameters: GameParameters,
    ) -> Result<()> {
        let max_players = game_parameters.max_players;
        let mut game = Game::create_game(&self.db, game_id, &user, game_parameters).await?;
        if max_players == 1 {
            Game::start_game(&self.db, game_id).await?;
            game.is_started = true;
        }
        let (bc_tx, _bc_rx) = broadcast::channel(100);
        let (mp_tx, mp_rx) = mpsc::channel(100);
        let (ch_tx, ch_rx) = mpsc::channel(100);
        let handle = GameHandle {
            to_client: bc_tx.clone(),
            from_client: mp_tx,
            game_events: ch_tx,
            players: Vec::with_capacity(max_players as usize),
            max_players,
            owner: user.map(|u| u.id),
            start_time: None,
        };
        {
            let mut games = self.games.write().await;
            games.insert(game_id.to_string(), handle);
        }
        let self_clone = self.clone();
        let game_handler = GameHandler::new(game, self_clone, bc_tx, mp_rx, ch_rx);
        tokio::spawn(async move { game_handler.handle_game().await });
        Ok(())
    }

    pub async fn game_exists(&self, game_id: &str) -> bool {
        Game::get_game(&self.db, game_id)
            .await
            .ok()
            .flatten()
            .is_some()
    }

    pub async fn get_active_games(&self) -> Vec<SimpleGameWithPlayers> {
        self.active_cache
            .get_or_set(|| async {
                let game_ids = {
                    let games = self.games.read().await;
                    games
                        .iter()
                        .map(|gh| gh.0)
                        .cloned()
                        .collect::<Vec<String>>()
                };
                if game_ids.is_empty() {
                    return Vec::new();
                }
                Game::get_games_with_players_by_ids(&self.db, &game_ids)
                    .await
                    .unwrap_or_default()
            })
            .await
    }

    pub async fn get_recent_games(&self) -> Vec<SimpleGameWithPlayers> {
        self.recent_cache
            .get_or_set(|| async {
                Game::get_recent_games_with_players(&self.db, TimeDelta::hours(-1))
                    .await
                    .unwrap_or_default()
            })
            .await
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Game> {
        Game::get_game(&self.db, game_id)
            .await?
            .ok_or(anyhow!("Game does not exist"))
    }

    pub async fn get_game_log(&self, game_id: &str) -> Result<GameLog> {
        GameLog::get_log(&self.db, game_id)
            .await?
            .ok_or(anyhow!("Game does not exist"))
    }

    pub async fn get_players(&self, game_id: &str) -> Result<Vec<PlayerUser>> {
        Player::get_players(&self.db, game_id)
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_player_games_for_user(&self, user: &User) -> Result<Vec<PlayerGame>> {
        Player::get_player_games_for_user(&self.db, user, 100)
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
        let (start_time, game_events, to_client) = {
            let games = self.games.read().await;
            if !games.contains_key(game_id) {
                bail!("Game with id {game_id} doesn't exist")
            }
            let handle = games.get(game_id).unwrap();
            (
                handle.start_time,
                handle.game_events.clone(),
                handle.to_client.clone(),
            )
        };
        if let Some(dt) = start_time {
            let mut sender = ws_sender.lock().await;
            let start_time_msg =
                GameMessage::SyncTimer(Utc::now().signed_duration_since(dt).num_seconds() as usize)
                    .into_json();
            let _ = sender.send(Message::Text(start_time_msg)).await;
        };
        game_events
            .send(GameEvent::Viewer(ViewerHandle { ws_sender }))
            .await?;
        Ok(to_client.subscribe())
    }

    pub async fn play_game(
        &self,
        game_id: &str,
        user: &Option<User>,
        ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) -> Result<mpsc::Sender<String>> {
        let user_id = user.as_ref().map(|u| u.id);
        let display_name = user.as_ref().and_then(|u| u.display_name.as_ref());

        let (player_id, save_player, game_events, from_client) = {
            let mut games = self.games.write().await;
            if !games.contains_key(game_id) {
                bail!("Game with id {game_id} doesn't exist")
            }

            let handle = games.get_mut(game_id).unwrap();
            let found = handle
                .players
                .iter_mut()
                .find(|p| user_id.is_some() && p.user_id == user_id);

            let mut save_player = false;
            let player_id = match found {
                None => {
                    let player_id = handle.players.len();
                    if player_id >= handle.max_players as usize {
                        bail!("Game already has max players")
                    }
                    save_player = true;
                    handle.players.push(PlayerHandle {
                        user_id,
                        player_id,
                        display_name: FrontendUser::display_name_or_anon(
                            display_name,
                            user.is_some(),
                        ),
                        ws_sender: Arc::clone(&ws_sender),
                    });
                    player_id
                }
                Some(p) => {
                    p.ws_sender = Arc::clone(&ws_sender);
                    p.player_id
                }
            };

            (
                player_id,
                save_player,
                handle.game_events.clone(),
                handle.from_client.clone(),
            )
        };
        if save_player {
            Player::add_player(&self.db, game_id, user, &None, player_id as u8).await?;
        }
        {
            let mut send = ws_sender.lock().await;
            let msg = GameMessage::PlayerId(player_id);
            (send).send(Message::Text(msg.into_json())).await?;
        }
        game_events
            .send(GameEvent::Player(PlayerHandle {
                user_id,
                player_id,
                display_name: FrontendUser::display_name_or_anon(display_name, user.is_some()),
                ws_sender: Arc::clone(&ws_sender),
            }))
            .await?;
        Ok(from_client)
    }

    pub async fn start_game(&self, game_id: &str, user: &Option<User>) -> Result<()> {
        let sender = {
            let mut games = self.games.write().await;
            if !games.contains_key(game_id) {
                bail!("Game with id {game_id} doesn't exist")
            }
            let handle = games.get_mut(game_id).unwrap();
            if let Some(owner) = handle.owner {
                match user {
                    None => {
                        bail!("Owned game attempted to be started by guest")
                    }
                    Some(user) => {
                        if owner != user.id {
                            bail!("Owned game attempted to be started by non-owner")
                        }
                    }
                }
            }
            handle.game_events.clone()
        };
        sender.send(GameEvent::Start).await?;
        Game::start_game(&self.db, game_id).await?;
        Ok(())
    }

    pub async fn set_start_time(&self, game_id: &str) -> Result<DateTime<Utc>> {
        let now = Utc::now();
        {
            let mut games = self.games.write().await;
            if !games.contains_key(game_id) {
                bail!("Game with id {game_id} doesn't exist")
            }
            let handle = games.get_mut(game_id).unwrap();
            handle.start_time = Some(now);
        }
        Game::set_start_time(&self.db, game_id, now).await?;
        Ok(now)
    }

    async fn save_game(&self, game_id: &str, board: Board<PlayerCell>) -> Result<()> {
        Game::save_board(&self.db, game_id, board.into()).await?;
        Ok(())
    }

    async fn complete_game(
        &self,
        game_id: &str,
        final_board: Board<PlayerCell>,
        end_time: Option<DateTime<Utc>>,
        seconds: Option<i64>,
        timed_out: bool,
    ) -> Result<()> {
        Game::complete_game(
            &self.db,
            game_id,
            final_board.into(),
            end_time,
            seconds,
            timed_out,
        )
        .await?;
        {
            let mut games = self.games.write().await;
            games.remove(game_id);
        }
        Ok(())
    }

    async fn save_game_log(&self, game_id: &str, game_log: Vec<(Play, PlayOutcome)>) -> Result<()> {
        GameLog::save_log(&self.db, game_id, game_log).await?;
        Ok(())
    }

    async fn update_players(&self, game_id: &str, players: Vec<ClientPlayer>) -> Result<()> {
        Player::update_players(&self.db, game_id, players).await?;
        Ok(())
    }

    pub async fn was_playing(&self, game_id: &str, user: &Option<User>) -> bool {
        if user.is_none() {
            return false;
        }
        let games = self.games.read().await;
        if !games.contains_key(game_id) {
            return false;
        }
        let handle = games.get(game_id).unwrap();
        let user_id = user.as_ref().map(|u| u.id);
        handle.players.iter().any(|p| p.user_id == user_id)
    }
}

struct GameHandler {
    game: Game,
    game_manager: GameManager,
    broadcaster: broadcast::Sender<String>,
    receiver: mpsc::Receiver<String>,
    game_events: mpsc::Receiver<GameEvent>,
    player_handles: Vec<Option<PlayerHandle>>,
    minesweeper: Minesweeper,
}

impl GameHandler {
    fn new(
        game: Game,
        game_manager: GameManager,
        broadcaster: broadcast::Sender<String>,
        receiver: mpsc::Receiver<String>,
        game_events: mpsc::Receiver<GameEvent>,
    ) -> Self {
        let player_handles = vec![None; game.max_players as usize];
        let mut minesweeper = MinesweeperBuilder::new(MinesweeperOpts {
            rows: game.rows as usize,
            cols: game.cols as usize,
            num_mines: game.num_mines as usize,
        })
        .unwrap()
        .with_superclick()
        .with_log();
        if game.max_players > 1 {
            minesweeper = minesweeper.with_multiplayer(game.max_players as usize);
        }
        let minesweeper = minesweeper.init();
        Self {
            game,
            game_manager,
            broadcaster,
            receiver,
            game_events,
            player_handles,
            minesweeper,
        }
    }

    async fn handle_game(mut self) {
        let mut checks_interval = interval(Duration::from_secs(5));

        let mut first_play = false;
        let mut needs_save = false;
        let mut timed_out = false;
        let mut start_time = None;
        let mut last_action = Utc::now();

        loop {
            tokio::select! {
                Some(msg) = self.receiver.recv() => {
                    log::debug!("Message received {}: {}", self.game.game_id, msg);
                    let played = self.handle_message(&msg).await.is_some();
                    if played {
                        needs_save = true;
                    }
                    if played && !first_play {
                        first_play = true;
                        if let Ok(st) = self.game_manager.set_start_time(&self.game.game_id).await.map_err(|e| log::error!("Error setting start time: {e}")) {
                            start_time = Some(st)
                        }
                        let sync_msg = GameMessage::SyncTimer(0).into_json();
                        log::debug!("Sending sync_msg {:?}", sync_msg);
                        let _ = self.broadcaster.send(sync_msg);
                    }
                    last_action = Utc::now();
                    if self.minesweeper.is_over() {
                        break;
                    }
                },
                Some(event) = self.game_events.recv() => {
                    log::debug!("Game update received {}: {:?}", self.game.game_id, event);
                    self.handle_game_event(event).await;
                    last_action = Utc::now();
                }
                _ = checks_interval.tick() => {
                    log::debug!("Checking for game {}", self.game.game_id);
                    let now = Utc::now();
                    if let Some(st) = start_time {
                        if now.signed_duration_since(st).num_seconds() >= 999 {
                            log::debug!("Game over time {}", self.game.game_id);
                            break;
                        }
                    }
                    if now.signed_duration_since(last_action).num_seconds() >= 120 {
                        log::debug!("Game timed out {}", self.game.game_id);
                        timed_out = true;
                        break;
                    }
                    if needs_save {
                        self.save_game_state_nonblocking();
                        needs_save = false;
                    }
                },
            }
        }

        if needs_save {
            self.save_game_state().await;
        }
        let minesweeper = self.minesweeper.complete();
        let (end_time, seconds) = if let Some(st) = start_time {
            if !timed_out {
                let now = Utc::now();
                let seconds = 999.min(now.signed_duration_since(st).num_seconds());
                (Some(now), Some(seconds))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };
        let _ = self
            .game_manager
            .complete_game(
                &self.game.game_id,
                minesweeper.viewer_board_final(),
                end_time,
                seconds,
                timed_out,
            )
            .await
            .map_err(|e| log::error!("Error completing game: {e}"));
        if let Some(game_log) = minesweeper.get_log() {
            let _ = self
                .game_manager
                .save_game_log(&self.game.game_id, game_log)
                .await
                .map_err(|e| log::error!("Error saving game log: {e}"));
        }
    }

    fn handles_to_client_players(&self) -> Vec<Option<ClientPlayer>> {
        let current_top_score = self.minesweeper.current_top_score();
        self.player_handles
            .iter()
            .map(|item| {
                item.as_ref().map(|player| {
                    let player_score = self.minesweeper.player_score(player.player_id).unwrap_or(0);
                    ClientPlayer {
                        player_id: player.player_id,
                        username: player.display_name.to_owned(),
                        dead: self
                            .minesweeper
                            .player_dead(player.player_id)
                            .unwrap_or(false),
                        victory_click: self
                            .minesweeper
                            .player_victory_click(player.player_id)
                            .unwrap_or(false),
                        top_score: current_top_score
                            .map(|s| s == player_score)
                            .unwrap_or(false),
                        score: player_score,
                    }
                })
            })
            .collect()
    }

    async fn save_game_state(&self) {
        let players = self
            .handles_to_client_players()
            .into_iter()
            .flatten()
            .collect();
        log::debug!("Saving game - players: {:?}", &players);
        let _ = self
            .game_manager
            .update_players(&self.game.game_id, players)
            .await
            .map_err(|e| log::error!("Error updating players: {e}"));
        let _ = self
            .game_manager
            .save_game(&self.game.game_id, self.minesweeper.viewer_board())
            .await
            .map_err(|e| log::error!("Error saving game: {e}"));
    }

    fn save_game_state_nonblocking(&self) {
        let players = self
            .handles_to_client_players()
            .into_iter()
            .flatten()
            .collect();
        let game_id = self.game.game_id.clone();
        let board = self.minesweeper.viewer_board();
        let game_manager = self.game_manager.clone();
        log::debug!("Saving game - players: {:?}", &players);
        tokio::spawn(async move {
            let game_id = game_id;
            let _ = game_manager
                .update_players(&game_id, players)
                .await
                .map_err(|e| log::error!("Error updating players: {e}"));
            let _ = game_manager
                .save_game(&game_id, board)
                .await
                .map_err(|e| log::error!("Error saving game: {e}"));
        });
    }

    async fn handle_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::Player(player) => {
                let player_sender = Arc::clone(&player.ws_sender);
                let player_id = player.player_id;
                let player_board = self.minesweeper.player_board(player_id);
                self.player_handles[player_id] = Some(player);
                {
                    let mut player_sender = player_sender.lock().await;
                    let player_msg = GameMessage::GameState(player_board).into_json();
                    log::debug!("Sending player_msg {:?}", player_msg);
                    let _ = player_sender.send(Message::Text(player_msg)).await;
                }

                let players = self.handles_to_client_players();
                let players_msg = GameMessage::PlayersState(players).into_json();
                log::debug!("Sending players_msg {:?}", players_msg);
                let _ = self.broadcaster.send(players_msg);
            }
            GameEvent::Viewer(viewer) => {
                let viewer_board = self.minesweeper.viewer_board();
                {
                    let mut viewer_sender = viewer.ws_sender.lock().await;
                    let viewer_msg = GameMessage::GameState(viewer_board).into_json();
                    log::debug!("Sending viewer_msg {:?}", viewer_msg);
                    let _ = viewer_sender.send(Message::Text(viewer_msg)).await;
                    let players = self.handles_to_client_players();
                    let players_msg = GameMessage::PlayersState(players).into_json();
                    let _ = viewer_sender.send(Message::Text(players_msg)).await;
                }
            }
            GameEvent::Start => {
                self.game.is_started = true;
                let start_msg = GameMessage::GameStarted.into_json();
                let _ = self.broadcaster.send(start_msg);
            }
        }
    }

    async fn handle_message(&mut self, msg: &str) -> Option<()> {
        if !self.game.is_started {
            return None;
        }
        let play = serde_json::from_str::<ClientMessage>(msg).ok()?;
        let play = match play {
            ClientMessage::Play(p) => p,
            _ => return None,
        };
        if play.player > self.player_handles.len() {
            return None;
        }
        let player = if let Some(player) = &self.player_handles[play.player] {
            player
        } else {
            return None;
        };
        let outcome = self.minesweeper.play(play);
        let res = match outcome {
            Ok(res) => res,
            Err(e) => {
                let err_msg = GameMessage::Error(format!("{:?}", e)).into_json();
                {
                    let mut player_sender = player.ws_sender.lock().await;
                    let _ = player_sender.send(Message::Text(err_msg)).await;
                }
                return None;
            }
        };
        match res {
            PlayOutcome::Flag(flag) => {
                let flag_msg = GameMessage::PlayOutcome(PlayOutcome::Flag(flag)).into_json();
                {
                    let mut player_sender = player.ws_sender.lock().await;
                    let _ = player_sender.send(Message::Text(flag_msg)).await;
                }
                None
            }
            default => {
                let victory_click = matches!(default, PlayOutcome::Victory(_));
                let outcome_msg = GameMessage::PlayOutcome(default).into_json();
                let score = self.minesweeper.player_score(player.player_id).unwrap();
                let dead = self.minesweeper.player_dead(player.player_id).unwrap();
                let top_score = self.minesweeper.player_top_score(player.player_id).unwrap();
                let player_state = ClientPlayer {
                    player_id: player.player_id,
                    username: player.display_name.to_owned(),
                    dead,
                    victory_click,
                    top_score,
                    score,
                };
                let player_state_message = GameMessage::PlayerUpdate(player_state).into_json();
                let _ = self.broadcaster.send(outcome_msg);
                let _ = self.broadcaster.send(player_state_message);
                Some(())
            }
        }
    }
}
