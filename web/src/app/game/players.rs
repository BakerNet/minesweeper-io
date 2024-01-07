use std::{cell::RefCell, rc::Rc};

use super::{client::FrontendGame, GameInfo};

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use minesweeper::client::ClientPlayer;
use reqwasm::http::Request;
use serde::Serialize;
use wasm_bindgen::JsValue;

#[cfg(feature = "ssr")]
use crate::app::FrontendUser;
#[cfg(feature = "ssr")]
use crate::backend::game_manager::GameManager;

#[component]
pub fn Players() -> impl IntoView {
    let game_info = use_context::<Resource<String, Result<GameInfo, ServerFnError>>>()
        .expect("Game info context missing");

    let player_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactivePlayers game_info/> },
        false => view! { <ActivePlayers /> },
    };

    view! {
        <Suspense fallback=move || ()>
            {game_info
                .get()
                .map(|game_info| {
                    view! {
                        <ErrorBoundary fallback=|_| {
                            view! { <div class="error">"Unable to load players"</div> }
                        }>{move || { game_info.clone().map(player_view) }}</ErrorBoundary>
                    }
                })}

        </Suspense>
    }
}

#[component]
pub fn ActivePlayers() -> impl IntoView {
    let game = use_context::<Rc<RefCell<FrontendGame>>>().unwrap();
    let (player, players, game_id) = {
        let game = (*game).borrow();
        (game.player, game.players.clone(), game.game_id.clone())
    };
    let last_slot = *players.last().unwrap();
    let available_slots = move || last_slot().is_none() && player().is_none();
    view! {
        <Show when=available_slots fallback=move || view! { <h4>Scoreboard</h4> }>
            <JoinForm game_id=game_id.clone()/>
        </Show>
        <table>
            <tr>
                <th>Player</th>
                <th>Username</th>
                <th>Score</th>
            </tr>
            {players
                .iter()
                .enumerate()
                .map(move |(n, player)| {
                    view! { <ActivePlayer player_num=n player=*player/> }
                })
                .collect_view()}
        </table>
        <A href="..">Hide</A>
    }
}

#[server(GetPlayers, "/api")]
pub async fn get_players(game_id: String) -> Result<Vec<ClientPlayer>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    Ok(players
        .iter()
        .map(|p| ClientPlayer {
            player_id: p.player as usize,
            username: FrontendUser::display_name_or_anon(&p.display_name),
            dead: p.dead,
            score: p.score as usize,
        })
        .collect())
}

#[component]
pub fn InactivePlayers(game_info: GameInfo) -> impl IntoView {
    let (game_info, _) = create_signal((game_info.max_players, game_info.game_id));
    let players = create_resource(game_info, |game_info| async move {
        let players = get_players(game_info.1.clone()).await.ok();
        players.map(|pv| {
            let mut players = vec![None; game_info.0 as usize];
            pv.iter()
                .for_each(|p| players[p.player_id] = Some(p.clone()));
            players
        })
    });
    view! {
        <table>
            <tr>
                <th>Player</th>
                <th>Username</th>
                <th>Score</th>
            </tr>
            <Transition fallback=move || {
                view! {}
            }>
                {move || {
                    let players = players.get().flatten()?;
                    Some(
                        players
                            .iter()
                            .enumerate()
                            .map(|(i, player)| {
                                view! { <InactivePlayer player_num=i player=player.clone()/> }
                            })
                            .collect_view(),
                    )
                }}

            </Transition>
        </table>
        <A href="..">Hide</A>
    }
}

#[component]
fn ActivePlayer(player_num: usize, player: ReadSignal<Option<ClientPlayer>>) -> impl IntoView {
    let items = move || {
        if let Some(player) = player() {
            (
                format!("p-{}", player.player_id),
                player.username,
                player.score,
            )
        } else {
            (String::from(""), String::from("--------"), 0)
        }
    };
    view! {
        <tr class=items().0>
            <td>{player_num}</td>
            <td>{items().1}</td>
            <td>{items().2}</td>
        </tr>
    }
}

#[component]
fn InactivePlayer(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (class, username, score) = if let Some(player) = &player {
        (
            format!("p-{}", player.player_id),
            player.username.clone(),
            player.score,
        )
    } else {
        (String::from(""), String::from("--------"), 0)
    };
    view! {
        <tr class=class>
            <td>{player_num}</td>
            <td>{username}</td>
            <td>{score}</td>
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

#[derive(Clone)]
pub struct PlayFormError {
    err_msg: String,
}

impl From<anyhow::Error> for PlayFormError {
    fn from(value: anyhow::Error) -> Self {
        PlayFormError {
            err_msg: format!("{:?}", value),
        }
    }
}

// TODO - rework joining game
#[component]
fn JoinForm(game_id: String) -> impl IntoView {
    let input_element: NodeRef<html::Input> = create_node_ref();
    let (game_id, _) = create_signal(game_id);
    let join_game: Action<(String, String), Result<(), PlayFormError>> =
        create_action(move |(user, game_id): &(String, String)| {
            let user = user.to_owned();
            let game_id = game_id.to_owned();
            async move {
                if user.is_empty() {
                    return Err(PlayFormError {
                        err_msg: String::from("User can't be empty"),
                    });
                }
                let form_message = PlayForm {
                    game_id: game_id.to_owned(),
                    user: user.to_owned(),
                }
                .to_jsvalue();
                let res = Request::post("/api/play")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(form_message)
                    .send()
                    .await
                    .map_err(|e| PlayFormError {
                        err_msg: format!("{:?}", e),
                    })?;
                let id = res.text().await.map_err(|e| PlayFormError {
                    err_msg: format!("{:?}", e),
                })?;
                if res.status() != 200 {
                    return Err(PlayFormError { err_msg: id });
                }
                let game = use_context::<Rc<RefCell<FrontendGame>>>().unwrap();
                let id = id.parse::<usize>().map_err(|e| PlayFormError {
                    err_msg: format!("{:?}", e),
                })?;
                game.borrow().set_player.set(Some(id));
                Result::Ok(())
            }
        });
    let join_game_val = join_game.value();

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            join_game.dispatch((input_element.get().unwrap().value(), game_id.get()));
        }>

            <input type="text" ref=input_element placeholder="Username"/>
            <button type="submit">"Join Game"</button>
        </form>
        <div class="error">
            {move || {
                if let Some(Err(e)) = join_game_val() { e.err_msg } else { String::from("") }
            }}

        </div>
    }
}
