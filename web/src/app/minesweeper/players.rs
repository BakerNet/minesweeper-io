use anyhow::Result;
use leptos::either::*;
use leptos::prelude::*;
use std::sync::Arc;

use minesweeper_lib::client::ClientPlayer;

#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
use crate::{
    button_class,
    components::icons::{IconTooltip, Mine, Star, Trophy},
    player_class, player_icon_holder,
};

use super::client::FrontendGame;

#[component]
fn Scoreboard(children: Children) -> impl IntoView {
    let header_class = "border dark:border-slate-600 font-medium p-4 text-gray-900 dark:text-gray-200 bg-neutral-500/50";
    view! {
        <table class="border border-solid border-slate-400 border-collapse table-auto w-full max-w-xs text-sm text-center bg-neutral-200/80 dark:bg-neutral-800/80">
            <thead>
                <tr>
                    <th class=header_class>"Player"</th>
                    <th class=header_class>"Username"</th>
                    <th class=header_class>"Score"</th>
                </tr>
            </thead>
            <tbody>{children()}</tbody>
        </table>
    }
}

#[component]
pub fn ActivePlayers(
    players: Arc<Vec<ReadSignal<Option<ClientPlayer>>>>,
    top_score: ReadSignal<Option<usize>>,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    let players_view = players
        .iter()
        .enumerate()
        .map(move |(n, player)| {
            view! { <ActivePlayer player_num=n player=*player top_score /> }
        })
        .collect_view();
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200">{title}</h2>
            <Scoreboard>{players_view}</Scoreboard>
            {children()}
        </div>
    }
}

#[component]
pub fn PlayerButtons(game: StoredValue<FrontendGame>) -> impl IntoView {
    let start_game = ServerAction::<StartGame>::new();

    let FrontendGame {
        game_id,
        is_owner,
        has_owner,
        player_id,
        players,
        players_loaded,
        started,
        join_trigger,
        ..
    } = game.get_value();
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
        Effect::watch(
            players_loaded,
            move |loaded, _, prev| {
                if *loaded && prev.unwrap_or(true) {
                    log::debug!("join_trigger");
                    join_trigger.notify();
                }
                !*loaded
            },
            false,
        );
    }

    view! {
        <Show when=show_play fallback=move || ()>
            <PlayForm join_trigger />
        </Show>
        <Show when=show_start>
            <StartForm start_game game_id=game_id.to_string() />
        </Show>
    }
}

#[component]
pub fn InactivePlayers(players: Vec<Option<ClientPlayer>>, title: &'static str) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200">{title}</h2>
            <Scoreboard>

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
fn ActivePlayer(
    player_num: usize,
    player: ReadSignal<Option<ClientPlayer>>,
    top_score: ReadSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        {move || {
            let mut player = player.get();
            let top_score = top_score.get();
            if let Some(ts) = top_score {
                if let Some(player) = &mut player {
                    if player.top_score && player.score < ts {
                        player.top_score = false;
                    }
                }
            }
            view! { <PlayerRow player_num=player_num player /> }
        }}
    }
}

#[component]
fn PlayerRow(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (mut player_class, username, is_dead, victory_click, top_score, score) =
        if let Some(player) = player {
            (
                player_class!(player.player_id).to_owned(),
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
                    Either::Left(
                        view! {
                            <span class=player_icon_holder!("bg-red-600", true)>
                                <Mine />
                                <IconTooltip>"Dead"</IconTooltip>
                            </span>
                        },
                    )
                } else {
                    Either::Right(())
                }}
                {if top_score {
                    Either::Left(
                        view! {
                            <span class=player_icon_holder!("bg-green-800", true)>
                                <Trophy />
                                <IconTooltip>"Top Score"</IconTooltip>
                            </span>
                        },
                    )
                } else {
                    Either::Right(())
                }}
                {if victory_click {
                    Either::Left(
                        view! {
                            <span class=player_icon_holder!("bg-black", true)>
                                <Star />
                                <IconTooltip>"Victory Click"</IconTooltip>
                            </span>
                        },
                    )
                } else {
                    Either::Right(())
                }}

            </td>
            <td class="border border-slate-100 dark:border-slate-700 p-1">{score}</td>
        </tr>
    }
}

#[component]
fn PlayForm(join_trigger: Trigger) -> impl IntoView {
    let (show, set_show) = signal(true);

    let join_game = move || {
        join_trigger.notify();
        set_show(false);
    };

    view! {
        <Show
            when=show
            fallback=move || {
                view! { <div>"Joining..."</div> }
            }
        >
            <form
                on:submit=move |ev| {
                    ev.prevent_default();
                    join_game();
                }

                class="w-full max-w-xs h-8"
            >
                <button type="submit" class=button_class!("w-full max-w-xs h-8")>
                    "Play Game"
                </button>
            </form>
        </Show>
    }
}

#[server]
pub async fn start_game(game_id: String) -> Result<(), ServerFnError> {
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
fn StartForm(start_game: ServerAction<StartGame>, game_id: String) -> impl IntoView {
    view! {
        <ActionForm action=start_game attr:class="w-full max-w-xs h-8">
            <input type="hidden" name="game_id" value=game_id />
            <button
                type="submit"
                class=button_class!(
                    "w-full max-w-xs h-8",
                    "bg-green-700 hover:bg-green-800/90 text-white"
                )

                disabled=start_game.pending()
            >
                "Start Game"
            </button>
        </ActionForm>
    }
}
