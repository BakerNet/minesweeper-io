use chrono::{DateTime, Utc};
use leptos::{either::*, prelude::*};
use leptos_router::components::*;

use leptos_use::{use_interval, UseIntervalReturn};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::backend::GameManager;
#[cfg(feature = "ssr")]
use crate::models::game::SimpleGameWithPlayers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleGameInfo {
    game_id: String,
    rows: usize,
    cols: usize,
    num_mines: usize,
    max_players: u8,
    is_started: bool,
    is_completed: bool,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    num_players: u8,
}

#[cfg(feature = "ssr")]
impl From<SimpleGameWithPlayers> for SimpleGameInfo {
    fn from(value: SimpleGameWithPlayers) -> Self {
        SimpleGameInfo {
            game_id: value.game_id,
            rows: value.rows as usize,
            cols: value.cols as usize,
            num_mines: value.num_mines as usize,
            max_players: value.max_players,
            is_started: value.is_started,
            is_completed: value.is_completed,
            start_time: value.start_time,
            end_time: value.end_time,
            num_players: value.num_players,
        }
    }
}

#[server]
pub async fn get_active_games() -> Result<(Vec<SimpleGameInfo>, Vec<SimpleGameInfo>), ServerFnError>
{
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let other_games = game_manager.get_active_games().await;

    Ok(other_games
        .into_iter()
        .fold((Vec::new(), Vec::new()), |acc, game| {
            let mut multiplayer_not_started = acc.0;
            let mut other = acc.1;
            if game.max_players > 1 && game.num_players < game.max_players && !game.is_started {
                multiplayer_not_started.push(game.into());
            } else {
                other.push(game.into());
            }
            (multiplayer_not_started, other)
        }))
}

#[server]
pub async fn get_recent_games() -> Result<Vec<SimpleGameInfo>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let other_games = game_manager.get_recent_games().await;

    Ok(other_games.into_iter().map(SimpleGameInfo::from).collect())
}

#[component]
fn NoneCard(children: Children) -> impl IntoView {
    view! {
        <div class="flex justify-center items-center h-16 w-full col-span-full text-gray-900 dark:text-gray-200">
            {children()}
        </div>
    }
}

#[component]
fn GameCard(games: Vec<SimpleGameInfo>) -> impl IntoView {
    match games.len() {
        0 => Either::Left(view! { <NoneCard>"No games found"</NoneCard> }),
        count => Either::Right(
            games
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, game_info)| {
                    let offset = if i == 0 && count < 3 {
                        if count == 1 {
                            "sm:col-start-2"
                        } else {
                            "xl:col-start-2"
                        }
                    } else {
                        ""
                    };
                    view! { <GameSummary game_info style=offset.to_owned() /> }
                })
                .collect_view(),
        ),
    }
}

#[component]
pub fn ActiveGames() -> impl IntoView {
    let active_games = Resource::new(move || (), move |_| async { get_active_games().await });

    let UseIntervalReturn { counter, .. } = use_interval(2000);

    Effect::watch(
        counter,
        move |_, _, _| {
            active_games.refetch();
        },
        false,
    );

    view! {
        <div class="flex-1 flex flex-col items-center justify-center w-full max-w-4xl py-12 px-4 space-y-4 mx-auto">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Join Multiplayer Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2">
                <Transition>
                    {move || Suspend::new(async move {
                        active_games
                            .await
                            .map(|(gs, _)| view! { <GameCard games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Watch Active Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2">
                <Transition>
                    {move || Suspend::new(async move {
                        active_games
                            .await
                            .map(|(_, gs)| view! { <GameCard games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
        </div>
    }
}

#[component]
pub fn RecentGames() -> impl IntoView {
    let recent_games = Resource::new(move || (), move |_| async { get_recent_games().await });

    let UseIntervalReturn { counter, .. } = use_interval(2000);

    Effect::watch(
        counter,
        move |_, _, _| {
            recent_games.refetch();
        },
        false,
    );

    view! {
        <div class="flex-1 flex flex-col items-center justify-center w-full max-w-4xl py-12 px-4 space-y-4 mx-auto">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Recent Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2">
                <Transition>
                    {move || Suspend::new(async move {
                        recent_games
                            .await
                            .map(|gs| view! { <GameCard games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
        </div>
    }
}

#[component]
fn GameSummary(game_info: SimpleGameInfo, style: String) -> impl IntoView {
    let url = format!("/game/{}", game_info.game_id);
    let section_class =
        "flex justify-center items-center border border-slate-100 dark:border-slate-700 p-1";
    let time = if !game_info.is_started {
        EitherOf3::A(view! { <>"Not started"</> })
    } else if game_info.is_completed {
        match (game_info.start_time, game_info.end_time) {
            (Some(st), Some(et)) => {
                EitherOf3::C(view! { <>{et.signed_duration_since(st).num_seconds()} " seconds"</> })
            }
            _ => EitherOf3::B(view! { <>"Unknown"</> }),
        }
    } else {
        match game_info.start_time {
            None => EitherOf3::B(view! { <>"Unknown"</> }),
            Some(t) => EitherOf3::C(
                view! { <>{Utc::now().signed_duration_since(t).num_seconds()} " seconds"</> },
            ),
        }
    };
    view! {
        <A href=url attr:class=style>
            <div class="h-auto w-full grid grid-cols-2 text-gray-700 dark:text-gray-400 hover:bg-sky-800/30">
                <div class=format!(
                    "{} {}",
                    section_class,
                    "col-span-2 bg-neutral-500/50 text-gray-900 dark:text-gray-200",
                )>{game_info.game_id}</div>
                <div class=section_class>"Size"</div>
                <div class=section_class>{game_info.rows}" x "{game_info.cols}</div>
                <div class=section_class>"Players"</div>
                <div class=section_class>{game_info.num_players}" / "{game_info.max_players}</div>
                <div class=section_class>"Time"</div>
                <div class=section_class>{time}</div>
            </div>
        </A>
    }
}
