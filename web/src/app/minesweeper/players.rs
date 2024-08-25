use anyhow::Result;
use leptos::*;
use leptos_router::*;
use std::rc::Rc;

use minesweeper_lib::client::ClientPlayer;

#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
use crate::components::{
    button_class,
    icons::{player_icon_holder, IconTooltip, Mine, Star, Trophy},
    player_class,
};

use super::client::PlayersContext;

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
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Player"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Username"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Score"
                    </th>
                </tr>
            </thead>
            <tbody>{children()}</tbody>
        </table>
        {buttons()}
    }
}

#[component]
pub fn ActivePlayers(players_context: PlayersContext) -> impl IntoView {
    let start_game = create_server_action::<StartGame>();

    let PlayersContext {
        game_id,
        is_owner,
        has_owner,
        player_id,
        players,
        players_loaded,
        started,
        join_trigger,
    } = players_context;
    log::debug!("players: {players:?}");
    let num_players = players.len();
    let last_slot = *players.last().unwrap();
    let show_play = move || {
        players_loaded() && last_slot().is_none() && player_id().is_none() && num_players > 1
    };
    let show_start = move || {
        players_loaded()
            && (is_owner || (!has_owner && player_id().is_some()))
            && !started()
            && num_players > 1
    };

    if num_players == 1 {
        log::debug!("num players 1");
        create_effect(move |_| {
            if players_loaded() {
                log::debug!("join_trigger");
                join_trigger.notify();
            }
        });
    }

    let buttons = move || {
        let game_id = Rc::clone(&game_id);
        view! {
            <Show when=show_play fallback=move || ()>
                <PlayForm join_trigger />
            </Show>
            <Show when=show_start fallback=move || ()>
                <StartForm start_game game_id=game_id.to_string() />
            </Show>
        }
    };

    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">Players</h4>
            <Scoreboard buttons>
                {players
                    .into_iter()
                    .enumerate()
                    .map(move |(n, player)| {
                        view! { <ActivePlayer player_num=n player=player /> }
                    })
                    .collect_view()}
            </Scoreboard>
        </div>
    }
}

#[component]
pub fn InactivePlayers(players: Vec<Option<ClientPlayer>>) -> impl IntoView {
    let is_victory = players
        .iter()
        .filter_map(|cp| cp.as_ref())
        .any(|cp| cp.victory_click);
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">
                {if is_victory { "Complete" } else { "Game Over" }}
            </h4>
            <Scoreboard buttons=move || ()>

                {players
                    .into_iter()
                    .enumerate()
                    .map(|(i, player)| {
                        view! { <PlayerRow player_num=i player=player /> }
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
            view! { <PlayerRow player_num=player_num player=player() /> }
        }}
    }
}

#[component]
fn PlayerRow(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (mut player_class, username, is_dead, victory_click, top_score, score) =
        if let Some(player) = player {
            (
                player_class(player.player_id),
                player.username,
                player.dead,
                player.victory_click,
                player.top_score,
                player.score,
            )
        } else {
            (
                String::from(""),
                String::from("--------"),
                false,
                false,
                false,
                0,
            )
        };
    if !player_class.is_empty() {
        player_class += " text-black";
    } else {
        player_class = "text-slate-600 dark:text-slate-400".to_string();
    }

    view! {
        <tr class=player_class>
            <td class="border border-slate-100 dark:border-slate-700 p-1">{player_num}</td>
            <td class="border border-slate-100 dark:border-slate-700 p-1">
                {username}
                {if is_dead {
                    view! {
                        <span class=player_icon_holder("bg-red-600", true)>
                            <Mine />
                            <IconTooltip>"Dead"</IconTooltip>
                        </span>
                    }
                        .into_view()
                } else {
                    ().into_view()
                }}
                {if top_score {
                    view! {
                        <span class=player_icon_holder("bg-green-800", true)>
                            <Trophy />
                            <IconTooltip>"Top Score"</IconTooltip>
                        </span>
                    }
                        .into_view()
                } else {
                    ().into_view()
                }}
                {if victory_click {
                    view! {
                        <span class=player_icon_holder("bg-black", true)>
                            <Star />
                            <IconTooltip>"Victory Click"</IconTooltip>
                        </span>
                    }
                        .into_view()
                } else {
                    ().into_view()
                }}

            </td>
            <td class="border border-slate-100 dark:border-slate-700 p-1">{score}</td>
        </tr>
    }
}

#[component]
fn PlayForm(join_trigger: Trigger) -> impl IntoView {
    let (show, set_show) = create_signal(true);

    let join_game = move || {
        join_trigger.notify();
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
                        <button type="submit" class=button_class(Some("w-full max-w-xs h-8"), None)>
                            "Play Game"
                        </button>
                    </form>
                }
                    .into_view()
            } else {
                view! { <div>"Joining..."</div> }.into_view()
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
            <input type="hidden" name="game_id" value=game_id />
            <button
                type="submit"
                class=button_class(
                    Some("w-full max-w-xs h-8"),
                    Some("bg-green-700 hover:bg-green-800/90 text-white"),
                )

                disabled=start_game.pending()
            >
                "Start Game"
            </button>
        </ActionForm>
    }
}
