use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

use anyhow::{anyhow, bail, Result};
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    client::{ClientPlayer, Play},
    game::{Action, Minesweeper, PlayOutcome},
    GameMessage,
};
use tokio::sync::broadcast;

struct Game {
    // We require unique usernames. This tracks which usernames have been taken.
    users: RwLock<Vec<String>>,
    // Channel used to send messages to all connected clients.
    tx: broadcast::Sender<String>,
    minesweeper: Mutex<Minesweeper>,
    num_players: usize,
}

impl Game {
    fn player(&self, username: &str) -> Option<usize> {
        let players = self.users.read().unwrap();
        players.iter().position(|s| s == username)
    }

    fn client_player(&self, player_id: usize) -> Result<ClientPlayer> {
        let (score, dead) = {
            let game = self.minesweeper.lock().unwrap();
            let score = game.player_score(player_id)?;
            let dead = game.player_dead(player_id)?;
            (score, dead)
        };
        Ok(ClientPlayer {
            player_id,
            username: self.users.read().unwrap()[player_id].clone(),
            dead,
            score,
        })
    }

    fn add_user(&mut self, username: &str) -> Result<usize> {
        if self.player(username).is_some() {
            bail!("Player with username {} already exists", username)
        }
        let mut users = self.users.write().unwrap();
        let player_id = users.len();
        if player_id >= self.num_players {
            bail!("Tried to join full game")
        }
        users.push(username.to_string());
        Ok(player_id)
    }

    fn users(&self) -> Vec<(usize, String)> {
        let users = self.users.read().unwrap();
        users
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.to_owned()))
            .collect::<Vec<_>>()
    }

    fn players_state(&self) -> Vec<Option<ClientPlayer>> {
        self.users()
            .iter()
            .map(|(id, _)| self.client_player(*id).ok())
            .collect()
    }

    fn game_state(&self) -> Vec<Vec<PlayerCell>> {
        let minesweeper = self.minesweeper.lock().unwrap();
        minesweeper.viewer_board()
    }

    fn player_game_state(&self, player: usize) -> Result<Vec<Vec<PlayerCell>>> {
        let minesweeper = self.minesweeper.lock().unwrap();
        Ok(minesweeper.player_board(player))
    }

    fn play(
        &mut self,
        player: usize,
        action: Action,
        cell_point: BoardPoint,
    ) -> Result<PlayOutcome> {
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

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
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
        let users = RwLock::new(Vec::new());
        let (tx, _rx) = broadcast::channel(100);
        let minesweeper = Mutex::new(Minesweeper::init_game(16, 30, 99, 8).unwrap());
        games.insert(
            id.to_string(),
            Game {
                users,
                tx,
                minesweeper,
                num_players: 8,
            },
        );
        Ok(())
    }

    pub fn users(&self, id: &str) -> Result<Vec<(usize, String)>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok(game.users())
    }

    pub fn game_exists(&self, id: &str) -> bool {
        let games = self.games.read().unwrap();
        let game = games.get(id);
        game.is_some()
    }

    pub fn join_game(
        &self,
        id: &str,
    ) -> Result<(broadcast::Receiver<String>, Vec<Vec<PlayerCell>>)> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok((game.tx.subscribe(), game.game_state()))
    }

    pub fn play_game(&self, id: &str, username: &str) -> Result<usize> {
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        let player_id = game.add_user(username)?;
        let player = game.client_player(player_id)?;
        let message = serde_json::to_string(&GameMessage::PlayerUpdate(player))?;
        let _ = game.tx.send(message); // Don't care if send fails
        Ok(player_id)
    }

    pub fn players_state(&self, id: &str) -> Result<Vec<Option<ClientPlayer>>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok(game.players_state())
    }

    pub fn game_state(&self, id: &str) -> Result<Vec<Vec<PlayerCell>>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        Ok(game.game_state())
    }

    pub fn player_game_state(&self, id: &str, player: usize) -> Result<Vec<Vec<PlayerCell>>> {
        let games = self.games.read().unwrap();
        let game = games.get(id).ok_or(game_err(id))?;
        game.player_game_state(player)
    }

    pub fn play(
        &self,
        id: &str,
        player: usize,
        action: Action,
        point: BoardPoint,
    ) -> Result<usize> {
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        let res = game.play(player, action, point)?;
        let res_json = serde_json::to_string(&res)?;
        game.tx.send(res_json).map_err(|e| anyhow!("{:?}", e))
    }

    pub fn handle_message(&self, id: &str, msg: &str) -> Result<Option<String>> {
        let play = serde_json::from_str::<Play>(msg)?;
        let mut games = self.games.write().unwrap();
        let game: &mut Game = games.get_mut(id).ok_or(game_err(id))?;
        let res = game.play(play.player, play.action, play.point)?;
        match res {
            PlayOutcome::Flag(flag) => {
                let message =
                    serde_json::to_string(&GameMessage::PlayOutcome(PlayOutcome::Flag(flag)))?;
                Ok(Some(message))
            }
            default => {
                let outcome = serde_json::to_string(&GameMessage::PlayOutcome(default))?;
                game.tx.send(outcome)?;
                Ok(None)
            }
        }
    }
}
