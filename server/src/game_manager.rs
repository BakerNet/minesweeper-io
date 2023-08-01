use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

use anyhow::{anyhow, bail, Result};
use axum::extract::ws::Message;
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    game::{Action, Minesweeper, PlayOutcome},
};
use tokio::sync::broadcast;

struct Game {
    // We require unique usernames. This tracks which usernames have been taken.
    user_set: RwLock<HashMap<String, usize>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
    minesweeper: Mutex<Minesweeper>,
    num_players: usize,
}

impl Game {
    fn player(&self, user: &str) -> Result<usize> {
        let players = self.user_set.read().unwrap();
        let player = players
            .get(user)
            .ok_or(anyhow!("User {user} doesn't exist"));
        player.copied()
    }

    fn add_user(&mut self, user: &str) -> Result<usize> {
        let mut users = self.user_set.write().unwrap();
        let player_id = user.len();
        if player_id >= self.num_players {
            bail!("Tried to join full game")
        }
        users.insert(user.to_string(), player_id);
        Ok(player_id)
    }

    fn game_state(&self) -> Vec<Vec<PlayerCell>> {
        let minesweeper = self.minesweeper.lock().unwrap();
        minesweeper.viewer_board()
    }

    fn player_state(&self, user: &str) -> Result<Vec<Vec<PlayerCell>>> {
        let player = self.player(user)?;
        let minesweeper = self.minesweeper.lock().unwrap();
        Ok(minesweeper.player_board(player))
    }

    fn play(&mut self, user: &str, action: Action, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let player = self.player(user)?;
        let mut minesweeper = self.minesweeper.lock().unwrap();
        minesweeper.play(player, action, cell_point)
    }
}

pub struct GameManager {
    games: RwLock<HashMap<String, Game>>,
}

fn game_err(id: &str) -> anyhow::Error {
    anyhow!("Game wiht id {id} doesn't exist")
}

impl GameManager {
    pub fn new() -> Self {
        GameManager {
            games: RwLock::new(HashMap::new()),
        }
    }

    pub fn new_game(&self, id: &str) -> Result<()> {
        let mut games = self.games.write().unwrap();
        if games.contains_key(id) {
            bail!("Game with id {id} already exists")
        }
        let user_set = RwLock::new(HashMap::new());
        let (tx, _rx) = broadcast::channel(100);
        let minesweeper = Mutex::new(Minesweeper::init_game(30, 16, 99, 8).unwrap());
        games.insert(
            id.to_string(),
            Game {
                user_set,
                tx,
                minesweeper,
                num_players: 8,
            },
        );
        Ok(())
    }

    pub fn game_exists(&self, id: &str) -> bool {
        let games = self.games.read().unwrap();
        let game = games.get(id);
        game.is_some()
    }

    pub fn join_game(&self, id: &str) -> Result<broadcast::Receiver<String>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok(game.tx.subscribe())
    }

    pub fn play_game(&self, id: &str, user: &str) -> Result<()> {
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        game.add_user(user).map(|_| ())
    }

    pub fn game_state(&self, id: &str) -> Result<Vec<Vec<PlayerCell>>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok(game.game_state())
    }

    pub fn player_game_state(&self, id: &str, user: &str) -> Result<Vec<Vec<PlayerCell>>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        game.player_state(user)
    }

    pub fn play(&self, id: &str, user: &str, action: Action, point: BoardPoint) -> Result<usize> {
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        let res = game.play(user, action, point)?;
        let res_json = serde_json::to_string(&res)?;
        game.tx.send(res_json).map_err(|e| anyhow!("{:?}", e))
    }

    pub fn handle_message(&self, id: &str, msg: &str) -> Result<()> {
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        game.tx.send(msg.to_string())?;
        Ok(())
    }
}
