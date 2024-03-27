use super::client::PlayersContext;

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use minesweeper_lib::client::ClientPlayer;
use serde::Serialize;

#[cfg(feature = "ssr")]
use crate::backend::{game_manager::GameManager, users::AuthSession};
use crate::components::{button_class, icons::Mine};

pub fn player_class(player: usize) -> String {
    String::from(match player {
        0 => "bg-cyan-200",
        1 => "bg-indigo-200",
        2 => "bg-fuchsia-200",
        3 => "bg-orange-200",
        4 => "bg-lime-200",
        5 => "bg-teal-200",
        6 => "bg-blue-200",
        7 => "bg-purple-200",
        8 => "bg-rose-200",
        9 => "bg-yellow-200",
        10 => "bg-emerald-200",
        11 => "bg-sky-200",
        _ => "",
    })
}

#[component]
fn Scoreboard<F, IV>(children: Children, buttons: F) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    view! {
        <table class="border border-solid border-slate-400 border-collapse table-auto w-full max-w-xs text-sm text-center">
            <thead>
                <tr>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Player
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Username
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Score
                    </th>
                </tr>
            </thead>
            <tbody>{children()}</tbody>
        </table>
        {buttons()}
    }
}

#[component]
pub fn ActivePlayers() -> impl IntoView {
    let players_ctx = expect_context::<PlayersContext>();
    let start_game = create_server_action::<StartGame>();

    let (player_id, players, loaded, started, join_trigger) = {
        (
            players_ctx.player_id,
            players_ctx.players.clone(),
            players_ctx.players_loaded,
            players_ctx.started,
            players_ctx.join_trigger,
        )
    };
    let num_players = players.len();
    let last_slot = *players.last().unwrap();
    let show_play =
        move || loaded() && last_slot().is_none() && player_id().is_none() && num_players > 1;
    let show_start = move || {
        loaded()
            && (players_ctx.is_owner || (!players_ctx.has_owner && player_id().is_some()))
            && !started()
    };

    if num_players == 1 {
        create_effect(move |_| {
            join_trigger.notify();
        });
    }

    let buttons = move || {
        let game_id = players_ctx.game_id.clone();
        view! {
            <Show when=show_play fallback=move || ()>
                <PlayForm/>
            </Show>
            <Show when=show_start fallback=move || ()>
                <StartForm start_game game_id=game_id.to_string()/>
            </Show>
        }
    };

    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">Players</h4>
            <Scoreboard buttons>
                {players
                    .iter()
                    .enumerate()
                    .map(move |(n, &player)| {
                        view! { <ActivePlayer player_num=n player=player/> }
                    })
                    .collect_view()}
            </Scoreboard>
        </div>
    }
}

#[component]
pub fn InactivePlayers(players: Vec<Option<ClientPlayer>>) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">Game Over</h4>
            <Scoreboard buttons=move || ()>

                {players
                    .iter()
                    .enumerate()
                    .map(|(i, player)| {
                        view! { <PlayerRow player_num=i player=player.clone()/> }
                    })
                    .collect_view()}

            </Scoreboard>
        </div>
    }
}

#[component]
fn ActivePlayer(player_num: usize, player: ReadSignal<Option<ClientPlayer>>) -> impl IntoView {
    view! {
        {move || {
            view! { <PlayerRow player_num=player_num player=player()/> }
        }}
    }
}

#[component]
fn PlayerRow(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (mut player_class, username, is_dead, score) = if let Some(player) = &player {
        (
            player_class(player.player_id),
            player.username.clone(),
            player.dead,
            player.score,
        )
    } else {
        (String::from(""), String::from("--------"), false, 0)
    };
    if !player_class.is_empty() {
        player_class += " text-black";
    } else {
        player_class = "text-slate-600 dark:text-slate-400".to_string();
    }

    view! {
        <tr class=player_class>
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">{player_num}</td>
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">
                {username}
                {if is_dead {
                    view! {
                        <span class="inline-block align-text-top bg-red-600 h-4 w-4">
                            <Mine/>
                        </span>
                    }
                        .into_view()
                } else {
                    ().into_view()
                }}

            </td>
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">{score}</td>
        </tr>
    }
}

#[derive(Serialize, Debug)]
pub struct PlayForm {
    game_id: String,
    user: String,
}

#[component]
fn PlayForm() -> impl IntoView {
    let players_ctx = expect_context::<PlayersContext>();
    let (trigger_join, _) = create_signal(players_ctx.join_trigger);
    let (show, set_show) = create_signal(true);

    let join_game = move || {
        trigger_join().notify();
        set_show(false);
    };

    view! {
        {move || {
            if show() {
                view! {
                    <form
                        on:submit=move |ev| {
                            ev.prevent_default();
                            join_game();
                        }

                        class="w-full max-w-xs h-8"
                    >
                        <button type="submit" class=button_class(Some("w-full w-max-xs h-8"), None)>
                            "Play Game"
                        </button>
                    </form>
                }
                    .into_view()
            } else {
                view! { <div>Joining...</div> }.into_view()
            }
        }}
    }
}

#[server(StartGame, "/api")]
async fn start_game(game_id: String) -> Result<(), ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    game_manager
        .start_game(&game_id, &auth_session.user)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[component]
fn StartForm(
    start_game: Action<StartGame, Result<(), ServerFnError>>,
    game_id: String,
) -> impl IntoView {
    view! {
        <ActionForm action=start_game class="w-full max-w-xs h-8">
            <input type="hidden" name="game_id" value=game_id/>
            <button
                type="submit"
                class=button_class(
                    Some("w-full w-max-xs h-8"),
                    Some("bg-green-700 hover:bg-green-800/90 text-white"),
                )
                disabled=move || start_game.pending()
            >
                "Start Game"
            </button>
        </ActionForm>
    }
}
