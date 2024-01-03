#![allow(dead_code)]
use anyhow::{bail, Result};
use minesweeper::{game::Minesweeper, GameMessage};
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::RwLock};
use tokio::sync::{broadcast, mpsc};

use crate::models::{
    game::{Game, Player, PlayerUser},
    user::User,
};

pub struct GameManager {
    games: RwLock<HashMap<String, (broadcast::Sender<String>, mpsc::Sender<String>)>>,
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            games: RwLock::new(HashMap::new()),
        }
    }

    pub async fn new_game(
        &self,
        db: SqlitePool,
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
            Game::create_game(&db, game_id, user, rows, cols, num_mines, max_players).await?;
        let (bc_tx, _bc_rx) = broadcast::channel(100);
        let (mp_tx, mp_rx) = mpsc::channel(100);
        games.insert(game_id.to_string(), (bc_tx.clone(), mp_tx));
        tokio::spawn(async move { handle_game(game, db, bc_tx, mp_rx) });
        Ok(())
    }

    pub fn join_game(
        &self,
        game_id: &str,
    ) -> Result<(broadcast::Receiver<String>, mpsc::Sender<String>)> {
        let games = self.games.read().unwrap();
        if !games.contains_key(game_id) {
            bail!("Game with id {game_id} doesn't exist")
        }
        let (bc, mp_tx) = games.get(game_id).unwrap();
        Ok((bc.subscribe(), mp_tx.clone()))
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
    let mut players: Vec<PlayerUser> = Vec::new();
    todo!()
}
