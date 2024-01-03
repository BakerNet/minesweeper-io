#![allow(dead_code)]
use anyhow::{bail, Result};
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use minesweeper::{game::Minesweeper, GameMessage};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::{broadcast, mpsc};

use crate::models::{
    game::{Game, Player, PlayerUser},
    user::User,
};

struct GameHandle {
    to_client: broadcast::Sender<String>,
    from_client: mpsc::Sender<String>,
    players: Vec<(String, Arc<Mutex<SplitSink<WebSocket, Message>>>)>,
    max_players: u8,
}

pub struct GameManager {
    db: SqlitePool,
    games: RwLock<HashMap<String, GameHandle>>,
}

impl GameManager {
    pub fn new(db: SqlitePool) -> Self {
        GameManager {
            db,
            games: RwLock::new(HashMap::new()),
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
        let mut games = self.games.write().unwrap();
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

    pub fn join_game(&self, game_id: &str) -> Result<broadcast::Receiver<String>> {
        let games = self.games.read().unwrap();
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get(game_id).unwrap();
        Ok(handle.to_client.subscribe())
    }

    pub async fn play_gamme(
        &self,
        game_id: &str,
        user: &User,
        ws_sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    ) -> Result<mpsc::Sender<String>> {
        let mut games = self.games.write().unwrap();
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let handle = games.get_mut(game_id).unwrap();
        if handle.players.len() >= handle.max_players as usize {
            bail!("Game already has max players")
        }
        let _player =
            Player::add_player(&self.db, game_id, user, handle.players.len() as u8).await?;
        handle
            .players
            .push((user.display_name_or_anon(), ws_sender));
        Ok(handle.from_client.clone())
    }
}

async fn handle_game(
    game: Game,
    db: SqlitePool,
    broadcaster: broadcast::Sender<String>,
    receiver: mpsc::Receiver<String>,
) -> () {
    let mut minesweeper = Minesweeper::init_game(
        game.rows as usize,
        game.cols as usize,
        game.num_mines as usize,
        game.max_players as usize,
    )
    .unwrap();

    todo!()
}
