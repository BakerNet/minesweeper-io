use leptos::either::*;
use leptos::prelude::*;

use minesweeper_lib::client::ClientPlayer;

use crate::{
    icons::{IconTooltip, Mine, Star, Trophy},
    player_class, player_icon_holder,
};

#[component]
fn Scoreboard(children: Children) -> impl IntoView {
    let header_class = "border dark:border-slate-600 font-medium p-2 text-gray-900 dark:text-gray-200 bg-neutral-500/50";
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
    players: impl IntoIterator<Item = ReadSignal<Option<ClientPlayer>>>,
    top_score: ReadSignal<Option<usize>>,
    children: Children,
) -> impl IntoView {
    let players_view = players
        .into_iter()
        .enumerate()
        .map(move |(n, player)| {
            view! { <ActivePlayer player_num=n player=player top_score /> }
        })
        .collect_view();
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <Scoreboard>{players_view}</Scoreboard>
            {children()}
        </div>
    }
}

#[component]
pub fn InactivePlayers(players: Vec<Option<ClientPlayer>>) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
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
