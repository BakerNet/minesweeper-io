use std::{cell::RefCell, rc::Rc};

use crate::game::client::FrontendGame;

use anyhow::{bail, Result};
use leptos::*;
use leptos_router::*;
use minesweeper::client::ClientPlayer;
use reqwasm::http::Request;
use serde::Serialize;
use wasm_bindgen::JsValue;

#[component]
pub fn Players(cx: Scope) -> impl IntoView {
    let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
    let (player, players, game_id) = {
        let game = (*game).borrow();
        (game.player, game.players.clone(), game.game_id.clone())
    };
    let last_slot = *players.last().unwrap();
    let available_slots = move || last_slot().is_none() && player().is_none();
    view! { cx,
        <Show when=available_slots fallback=move |_| view!{cx, <h4>Scoreboard</h4>}>
            <JoinForm game_id=game_id.clone() />
        </Show>
        <table>
        <tr><th>Player</th><th>Username</th><th>Score</th></tr>
        {players.iter().enumerate().map(move |(n, player)|{
            view! { cx, <Player player_num=n player=*player /> }
        }).collect_view(cx)}
        </table>
        <A href="..">Hide</A>
    }
}

#[component]
fn Player(cx: Scope, player_num: usize, player: ReadSignal<Option<ClientPlayer>>) -> impl IntoView {
    view! {cx,
        <tr>
            <td>{player_num}</td>
            <td>{move || {
                if let Some(player) = player() {
                    player.username
                } else {
                    String::from("--------")
                }
            }}</td>
            <td>{move || {
                if let Some(player) = player() {
                    player.score
                } else {
                    0
                }
            }}</td>
        </tr>
    }
}

#[derive(Serialize, Debug)]
pub struct PlayForm {
    game_id: String,
    user: String,
}

impl PlayForm {
    fn to_jsvalue(&self) -> JsValue {
        JsValue::from_str(&format!("user={}&game_id={}", self.user, self.game_id))
    }
}

#[component]
fn JoinForm(cx: Scope, game_id: String) -> impl IntoView {
    let input_element: NodeRef<html::Input> = create_node_ref(cx);
    let (game_id, _) = create_signal(cx, game_id);
    let join_game: Action<(String, String), Result<()>> =
        create_action(cx, move |(user, game_id): &(String, String)| {
            let user = user.to_owned();
            let game_id = game_id.to_owned();
            async move {
                if user.is_empty() {
                    bail!("User can't be empty")
                }
                let form_message = PlayForm {
                    game_id: game_id.to_owned(),
                    user: user.to_owned(),
                }
                .to_jsvalue();
                let id = Request::post("http://127.0.0.1:3000/api/play")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(form_message)
                    .send()
                    .await?
                    .text()
                    .await?;
                let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
                let id = id.parse::<usize>()?;
                game.borrow().set_player.set(Some(id));
                Result::Ok(())
            }
        });

    view! {cx,
        <form
            on:submit=move |ev| {
                ev.prevent_default(); // don't reload the page...
                join_game.dispatch((input_element.get().unwrap().value(), game_id.get()));
            }
         >
            <input type="text" ref=input_element placeholder="Username" />
            <button type="submit">"Join Game"</button>
        </form>
    }
}
