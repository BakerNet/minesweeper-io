use chrono::{DateTime, Utc};
use leptos::{either::*, prelude::*};
use leptos_router::components::*;

use leptos_use::{use_interval, UseIntervalReturn};
use serde::{Deserialize, Serialize};

use game_ui::{GameMode, GameSettings};

#[cfg(feature = "ssr")]
use game_manager::{models::SimpleGameWithPlayers, GameManager};

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
    timed_out: Option<bool>,
    num_players: u8,
    seconds: Option<i64>,
    top_score: Option<i64>,
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
            timed_out: value.timed_out,
            num_players: value.num_players,
            seconds: value.seconds,
            top_score: value.top_score,
        }
    }
}

#[server]
pub async fn get_active_games() -> Result<Vec<SimpleGameInfo>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let active_games = game_manager.get_active_games().await;

    Ok(active_games.into_iter().map(SimpleGameInfo::from).collect())
}

#[server]
pub async fn get_recent_games() -> Result<Vec<SimpleGameInfo>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let other_games = game_manager.get_recent_games().await;

    Ok(other_games.into_iter().map(SimpleGameInfo::from).collect())
}

#[component]
pub fn ActiveGames() -> impl IntoView {
    let title_class = "text-4xl my-4 text-gray-900 dark:text-gray-200";
    let section_class = "grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2";

    let active_games = Resource::new(
        move || (),
        move |_| async {
            let games = get_active_games().await;
            games.map(|vec| {
                vec.into_iter()
                    .fold((Vec::new(), Vec::new(), Vec::new()), |acc, game| {
                        let mut multiplayer_joinable = acc.0;
                        let mut multiplayer_other = acc.1;
                        let mut singleplayer = acc.2;
                        if game.max_players > 1 && game.num_players < game.max_players {
                            multiplayer_joinable.push(game);
                        } else if game.max_players > 1 {
                            multiplayer_other.push(game);
                        } else {
                            singleplayer.push(game);
                        }
                        (multiplayer_joinable, multiplayer_other, singleplayer)
                    })
            })
        },
    );

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
            <h1 class=title_class>"Join Multiplayer Games"</h1>
            <div class=section_class>
                <Transition>
                    {move || Suspend::new(async move {
                        active_games
                            .await
                            .map(|(gs, _, _)| view! { <GameCards games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
            <h1 class=title_class>"Watch Multiplayer Games"</h1>
            <div class=section_class>
                <Transition>
                    {move || Suspend::new(async move {
                        active_games
                            .await
                            .map(|(_, gs, _)| view! { <GameCards games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
            <h1 class=title_class>"Watch Active Games"</h1>
            <div class=section_class>
                <Transition>
                    {move || Suspend::new(async move {
                        active_games
                            .await
                            .map(|(_, _, gs)| view! { <GameCards games=gs /> }.into_any())
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
    let title_class = "text-4xl my-4 text-gray-900 dark:text-gray-200";
    let section_class = "grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2";

    let recent_games = Resource::new(
        move || (),
        move |_| async {
            let games = get_recent_games().await;
            games.map(|vec| {
                vec.into_iter().fold((Vec::new(), Vec::new()), |acc, game| {
                    let mut multiplayer = acc.0;
                    let mut singleplayer = acc.1;
                    if game.max_players > 1 {
                        multiplayer.push(game);
                    } else {
                        singleplayer.push(game);
                    }
                    (multiplayer, singleplayer)
                })
            })
        },
    );

    let UseIntervalReturn { counter, .. } = use_interval(5000);

    Effect::watch(
        counter,
        move |_, _, _| {
            recent_games.refetch();
        },
        false,
    );

    view! {
        <div class="flex-1 flex flex-col items-center justify-center w-full max-w-4xl py-12 px-4 space-y-4 mx-auto">
            <h1 class=title_class>"Recent Multiplayer Games"</h1>
            <div class=section_class>
                <Transition>
                    {move || Suspend::new(async move {
                        recent_games
                            .await
                            .map(|(gs, _)| view! { <GameCards games=gs /> }.into_any())
                            .unwrap_or(
                                view! { <NoneCard>"Error fetching games"</NoneCard> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
            <h1 class=title_class>"Recent Singleplayer Games"</h1>
            <div class=section_class>
                <Transition>
                    {move || Suspend::new(async move {
                        recent_games
                            .await
                            .map(|(_, gs)| view! { <GameCards games=gs /> }.into_any())
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
fn NoneCard(children: Children) -> impl IntoView {
    view! {
        <div class="flex justify-center items-center h-16 w-full col-span-full text-gray-900 dark:text-gray-200">
            {children()}
        </div>
    }
}

#[component]
fn GameCards(games: Vec<SimpleGameInfo>) -> impl IntoView {
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
fn GameSummary(game_info: SimpleGameInfo, style: String) -> impl IntoView {
    let url = format!("/game/{}", game_info.game_id);
    let section_class =
        "flex justify-center items-center border border-slate-100 dark:border-slate-700 p-1";

    let game_mode = GameMode::from(GameSettings {
        rows: game_info.rows as i64,
        cols: game_info.cols as i64,
        num_mines: game_info.num_mines as i64,
        max_players: game_info.max_players as i64,
    });
    let mode = match game_mode {
        GameMode::ClassicBeginner | GameMode::ClassicIntermediate | GameMode::ClassicExpert => {
            game_mode.short_name().to_string()
        }
        GameMode::SmallMultiplayer => "Multi Small".to_string(),
        GameMode::LargeMultiplayer => "Multi Large".to_string(),
        GameMode::Custom => format!("Custom {}x{}", game_info.rows, game_info.cols),
    };

    let time = if game_info.start_time.is_none() {
        EitherOf4::A(view! { <span>"Not started"</span> })
    } else if game_info.timed_out.is_some_and(|to| to) {
        EitherOf4::B(view! { <span>"Timed out"</span> })
    } else if let Some(secs) = game_info.seconds {
        EitherOf4::C(view! { <span>{secs}" seconds"</span> })
    } else if game_info.is_completed {
        match (game_info.start_time, game_info.end_time) {
            (Some(st), Some(et)) => EitherOf4::C(
                view! { <span>{999.min(et.signed_duration_since(st).num_seconds())} " seconds"</span> },
            ),
            _ => EitherOf4::D(view! { <span>"Unknown"</span> }),
        }
    } else {
        EitherOf4::C(view! {
            <span>
                {Utc::now().signed_duration_since(game_info.start_time.unwrap()).num_seconds()}
                " seconds"
            </span>
        })
    };

    let score_header = if game_info.max_players == 1 {
        "Score"
    } else {
        "Top Score"
    };
    let top_score = if let Some(score) = game_info.top_score {
        match (game_mode, score) {
            (GameMode::ClassicBeginner, 71) => "Victory".to_string(),
            (GameMode::ClassicIntermediate, 216) => "Victory".to_string(),
            (GameMode::ClassicExpert, 381) => "Victory".to_string(),
            _ => {
                if game_info.start_time.is_some() {
                    format!("{score}")
                } else {
                    "N/A".to_string()
                }
            }
        }
    } else {
        "N/A".to_string()
    };

    view! {
        <A href=url attr:class=style>
            <div class="h-auto w-full grid grid-cols-[2fr_3fr] text-gray-700 dark:text-gray-400 hover:bg-sky-800/30 bg-neutral-200/80 dark:bg-neutral-800/80">
                <div class=format!(
                    "{} {}",
                    section_class,
                    "col-span-2 bg-neutral-500/50 text-gray-900 dark:text-gray-200 font-bold",
                )>{game_info.game_id}</div>
                <div class=section_class>"Mode"</div>
                <div class=section_class>{mode}</div>
                {if game_info.max_players != 1 {
                    Either::Left(
                        view! {
                            <div class=section_class>"Players"</div>
                            <div class=section_class>
                                {game_info.num_players}" / "{game_info.max_players}
                            </div>
                        },
                    )
                } else {
                    Either::Right(
                        view! {
                            <div class=section_class>"Time"</div>
                            <div class=section_class>{time}</div>
                        },
                    )
                }}
                <div class=section_class>{score_header}</div>
                <div class=section_class>{top_score}</div>
            </div>
        </A>
    }
}
