use chrono::{DateTime, Utc};
use leptos::prelude::*;
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
    num_players: u8,
}

#[server]
pub async fn get_active_games() -> Result<(Vec<SimpleGameInfo>, Vec<SimpleGameInfo>), ServerFnError>
{
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let mp_games = game_manager.get_multiplayer_not_started().await;
    let other_games = game_manager.get_other_games().await;

    let into_simple_game_info = move |game: SimpleGameWithPlayers| SimpleGameInfo {
        game_id: game.game_id,
        rows: game.rows as usize,
        cols: game.cols as usize,
        num_mines: game.num_mines as usize,
        max_players: game.max_players,
        is_started: game.is_started,
        is_completed: game.is_completed,
        start_time: game.start_time,
        num_players: game.num_players,
    };
    Ok((
        mp_games.into_iter().map(into_simple_game_info).collect(),
        other_games.into_iter().map(into_simple_game_info).collect(),
    ))
}

#[component]
pub fn ActiveGames() -> impl IntoView {
    let games = Resource::new(move || (), move |_| async { get_active_games().await });
    let none_class = "flex justify-center items-center h-16 w-full col-span-full text-gray-900 dark:text-gray-200";

    let UseIntervalReturn { counter, .. } = use_interval(5000);

    Effect::watch(
        counter,
        move |_, _, _| {
            games.refetch();
        },
        false,
    );

    let games_view = move |games: Vec<SimpleGameInfo>| match games.len() {
        0 => view! { <div class=none_class>"No games found"</div> }.into_any(),
        count => games
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
                view! { <ActiveGame game_info style=offset.to_owned() /> }
            })
            .collect_view()
            .into_any(),
    };

    view! {
        <div class="flex-1 flex flex-col items-center justify-center w-full max-w-4xl py-12 px-4 space-y-4 mx-auto">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Join Multiplayer Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2">
                <Transition>
                    {move || Suspend::new(async move {
                        games
                            .await
                            .map(|(gs, _)| games_view(gs))
                            .unwrap_or(
                                view! { <div class=none_class>"No games found"</div> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Watch Active Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full gap-2">
                <Transition>
                    {move || Suspend::new(async move {
                        games
                            .await
                            .map(|(_, gs)| games_view(gs))
                            .unwrap_or(
                                view! { <div class=none_class>"No games found"</div> }.into_any(),
                            )
                    })}
                </Transition>
            </div>
        </div>
    }
}

#[component]
fn ActiveGame(game_info: SimpleGameInfo, style: String) -> impl IntoView {
    let url = format!("/game/{}", game_info.game_id);
    let section_class =
        "flex justify-center items-center border border-slate-100 dark:border-slate-700 p-1";
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

            </div>
        </A>
    }
}
