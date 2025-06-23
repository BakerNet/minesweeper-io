use leptos::either::*;
use leptos::prelude::*;
use leptos_router::components::*;
use serde::{Deserialize, Serialize};

use game_ui::{
    icons::{IconTooltip, Mine, Star, Trophy},
    player_class, player_icon_holder, GameMode,
};

const PER_PAGE: usize = 100;

#[cfg(feature = "ssr")]
use game_manager::models::{
    GameModeFilter as BackendGameModeFilter, GameQueryParams,
    GameStatusFilter as BackendGameStatusFilter, SortBy as BackendSortBy,
    SortOrder as BackendSortOrder,
};
#[cfg(feature = "ssr")]
use game_manager::GameManager;
#[cfg(feature = "ssr")]
use game_ui::GameSettings;
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerGame {
    game_id: String,
    player: u8,
    dead: bool,
    victory_click: bool,
    top_score: bool,
    score: i64,
    start_time: Option<String>,
    game_time: Option<usize>,
    game_mode: GameMode,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameStatusFilter {
    All,
    Won,
    Lost,
    InProgress,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameModeFilter {
    All,
    Beginner,
    Intermediate,
    Expert,
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SortBy {
    Date,
    Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[cfg(feature = "ssr")]
impl From<GameModeFilter> for BackendGameModeFilter {
    fn from(filter: GameModeFilter) -> Self {
        match filter {
            GameModeFilter::All => BackendGameModeFilter::All,
            GameModeFilter::Beginner => BackendGameModeFilter::Beginner,
            GameModeFilter::Intermediate => BackendGameModeFilter::Intermediate,
            GameModeFilter::Expert => BackendGameModeFilter::Expert,
            GameModeFilter::Custom => BackendGameModeFilter::Custom,
        }
    }
}

#[cfg(feature = "ssr")]
impl From<GameStatusFilter> for BackendGameStatusFilter {
    fn from(filter: GameStatusFilter) -> Self {
        match filter {
            GameStatusFilter::All => BackendGameStatusFilter::All,
            GameStatusFilter::Won => BackendGameStatusFilter::Won,
            GameStatusFilter::Lost => BackendGameStatusFilter::Lost,
            GameStatusFilter::InProgress => BackendGameStatusFilter::InProgress,
        }
    }
}

#[cfg(feature = "ssr")]
impl From<SortBy> for BackendSortBy {
    fn from(sort: SortBy) -> Self {
        match sort {
            SortBy::Date => BackendSortBy::Date,
            SortBy::Duration => BackendSortBy::Duration,
        }
    }
}

#[cfg(feature = "ssr")]
impl From<SortOrder> for BackendSortOrder {
    fn from(order: SortOrder) -> Self {
        match order {
            SortOrder::Asc => BackendSortOrder::Asc,
            SortOrder::Desc => BackendSortOrder::Desc,
        }
    }
}

#[server]
pub async fn get_player_games(
    page: i64,
    mode_filter: GameModeFilter,
    status_filter: GameStatusFilter,
    sort_by: SortBy,
    sort_order: SortOrder,
) -> Result<Vec<PlayerGame>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let user = auth_session.user.ok_or(ServerFnError::new(
        "Cannot find player games when not logged in".to_string(),
    ))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    let params = GameQueryParams {
        page,
        limit: PER_PAGE as i64,
        mode_filter: mode_filter.into(),
        status_filter: status_filter.into(),
        sort_by: sort_by.into(),
        sort_order: sort_order.into(),
    };

    let games = game_manager
        .get_player_games_for_user(&user, &params)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(games
        .into_iter()
        .map(|pu| PlayerGame {
            game_id: pu.game_id,
            player: pu.player,
            dead: pu.dead,
            victory_click: pu.victory_click,
            top_score: pu.top_score,
            score: pu.score,
            start_time: pu
                .start_time
                .map(|dt| dt.date_naive().format("%Y-%m-%d").to_string()),
            game_time: match (pu.start_time, pu.end_time) {
                (Some(st), Some(et)) => {
                    Some(999.min(et.signed_duration_since(st).num_seconds() as usize))
                }
                _ => None,
            },
            game_mode: GameMode::from(GameSettings::new(
                pu.rows,
                pu.cols,
                pu.num_mines,
                pu.max_players.into(),
            )),
        })
        .collect())
}

#[component]
pub fn GameHistory() -> impl IntoView {
    // State management
    let (page, set_page) = signal(1i64);
    let (mode_filter, set_mode_filter) = signal(GameModeFilter::All);
    let (status_filter, set_status_filter) = signal(GameStatusFilter::All);
    let (sort_by, set_sort_by) = signal(SortBy::Date);
    let (sort_order, set_sort_order) = signal(SortOrder::Desc);

    let player_games = Resource::new(
        move || {
            (
                page.get(),
                mode_filter.get(),
                status_filter.get(),
                sort_by.get(),
                sort_order.get(),
            )
        },
        move |(page, mode_filter, status_filter, sort_by, sort_order)| async move {
            get_player_games(page, mode_filter, status_filter, sort_by, sort_order).await
        },
    );
    let td_class = "border border-slate-100 dark:border-slate-700 p-1";
    let header_class = "border dark:border-slate-600 font-medium p-4 text-gray-900 dark:text-gray-200 bg-neutral-500/50";

    let loading_row = move |num: usize| {
        let player_class = player_class!(0).to_owned() + " text-black";
        view! {
            <tr class=player_class>
                <td class=td_class>"Game "{num}</td>
                <td class=td_class></td>
                <td class=td_class></td>
                <td class=td_class>"Loading..."</td>
                <td class=td_class></td>
                <td class=td_class></td>
            </tr>
        }
    };
    let game_view = move |game: PlayerGame| {
        let player_class = player_class!(game.player as usize).to_owned() + " text-black";
        view! {
            <tr class=player_class>
                <td class=td_class>
                    <A
                        attr:class="text-sky-700 dark:text-sky-500 hover:text-sky-900 dark:hover:text-sky-400 font-medium"
                        href=format!("/game/{}", game.game_id)
                    >
                        {game.game_id}
                    </A>
                </td>
                <td class=td_class>{game.start_time}</td>
                <td class=td_class>{game.game_mode.long_name()}</td>
                <td class=td_class>{game.game_time}</td>
                <td class=td_class>
                    {if game.dead {
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
                    {if game.top_score {
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
                    {if game.victory_click {
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
                <td class=td_class>{game.score}</td>
            </tr>
        }
    };
    // Helper functions for sort controls
    let toggle_sort = move |new_sort_by: SortBy| {
        if sort_by.get() == new_sort_by {
            // Same column, toggle order
            set_sort_order.update(|order| {
                *order = match *order {
                    SortOrder::Asc => SortOrder::Desc,
                    SortOrder::Desc => SortOrder::Asc,
                }
            });
        } else {
            // Different column, set new sort and default to desc
            set_sort_by.set(new_sort_by);
            set_sort_order.set(SortOrder::Desc);
        }
        set_page.set(1); // Reset to first page when sorting changes
    };

    view! {
        <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200">"Game History"</h2>

        // Filters and Controls
        <div class="mb-4 space-y-4">
            <div class="flex flex-wrap gap-4 items-center">
                // Mode Filter
                <div class="flex items-center space-x-2">
                    <label class="text-sm font-medium text-gray-700 dark:text-gray-300">"Mode:"</label>
                    <select
                        class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let filter = match value.as_str() {
                                "Beginner" => GameModeFilter::Beginner,
                                "Intermediate" => GameModeFilter::Intermediate,
                                "Expert" => GameModeFilter::Expert,
                                "Custom" => GameModeFilter::Custom,
                                _ => GameModeFilter::All,
                            };
                            set_mode_filter.set(filter);
                            set_page.set(1);
                        }
                    >
                        <option value="All" selected=move || matches!(mode_filter.get(), GameModeFilter::All)>"All"</option>
                        <option value="Beginner" selected=move || matches!(mode_filter.get(), GameModeFilter::Beginner)>"Beginner"</option>
                        <option value="Intermediate" selected=move || matches!(mode_filter.get(), GameModeFilter::Intermediate)>"Intermediate"</option>
                        <option value="Expert" selected=move || matches!(mode_filter.get(), GameModeFilter::Expert)>"Expert"</option>
                        <option value="Custom" selected=move || matches!(mode_filter.get(), GameModeFilter::Custom)>"Custom"</option>
                    </select>
                </div>

                // Status Filter
                <div class="flex items-center space-x-2">
                    <label class="text-sm font-medium text-gray-700 dark:text-gray-300">"Status:"</label>
                    <select
                        class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            let filter = match value.as_str() {
                                "Won" => GameStatusFilter::Won,
                                "Lost" => GameStatusFilter::Lost,
                                "InProgress" => GameStatusFilter::InProgress,
                                _ => GameStatusFilter::All,
                            };
                            set_status_filter.set(filter);
                            set_page.set(1);
                        }
                    >
                        <option value="All" selected=move || matches!(status_filter.get(), GameStatusFilter::All)>"All"</option>
                        <option value="Won" selected=move || matches!(status_filter.get(), GameStatusFilter::Won)>"Won"</option>
                        <option value="Lost" selected=move || matches!(status_filter.get(), GameStatusFilter::Lost)>"Lost"</option>
                        <option value="InProgress" selected=move || matches!(status_filter.get(), GameStatusFilter::InProgress)>"In Progress"</option>
                    </select>
                </div>
            </div>

            // Pagination Controls
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-2">
                    <span class="text-sm text-gray-600 dark:text-gray-400">
                        "Page " {move || page.get()}
                        <Suspense>
                            {move || {
                                player_games.with(|games| {
                                    match games {
                                        Some(Ok(games)) => format!(" ({} games)", PER_PAGE.min(games.len())),
                                        _ => String::new(),
                                    }
                                })
                            }}
                        </Suspense>
                    </span>
                </div>
                <div class="flex space-x-2">
                    <button
                        class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
                        disabled=move || page.get() <= 1
                        on:click=move |_| set_page.update(|p| *p = (*p - 1).max(1))
                    >
                        "Previous"
                    </button>
                    <Transition fallback=move || view!{
                        <button class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed" disabled>"Next"</button>
                    }>
                        {move || {
                            player_games.with(|games| {
                                let disabled = if let Some(Ok(games)) = games {
                                    games.len() < PER_PAGE
                                } else {true};
                                view! {
                                    <button
                                        class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-300 disabled:cursor-not-allowed"
                                        disabled=disabled
                                        on:click=move |_| set_page.update(|p| *p += 1)
                                    >
                                        "Next"
                                    </button>
                                }
                            })
                        }}
                    </Transition>
                </div>
            </div>
        </div>

        <div class="max-w-full overflow-x-auto">
            <table class="border border-solid border-slate-400 border-collapse table-auto text-sm text-center bg-neutral-200/80 dark:bg-neutral-800/80">
                <thead>
                    <tr>
                        <th class=header_class>"Game"</th>
                        <th class=format!("{} cursor-pointer hover:bg-neutral-400/50", header_class)
                            on:click=move |_| toggle_sort(SortBy::Date)>
                            "Date "
                            {move || if matches!(sort_by.get(), SortBy::Date) {
                                match sort_order.get() {
                                    SortOrder::Desc => "↓",
                                    SortOrder::Asc => "↑",
                                }
                            } else {
                                ""
                            }}
                        </th>
                        <th class=header_class>"Game Mode"</th>
                        <th class=format!("{} cursor-pointer hover:bg-neutral-400/50", header_class)
                            on:click=move |_| toggle_sort(SortBy::Duration)>
                            "Duration "
                            {move || if matches!(sort_by.get(), SortBy::Duration) {
                                match sort_order.get() {
                                    SortOrder::Desc => "↓",
                                    SortOrder::Asc => "↑",
                                }
                            } else {
                                ""
                            }}
                        </th>
                        <th class=header_class>"Status"</th>
                        <th class=header_class>"Score"</th>
                    </tr>
                </thead>
                <tbody>
                    <Transition fallback=move || {
                        (0..5).map(loading_row).collect_view()
                    }>

                        {move || {
                            Suspend::new(async move {
                                player_games
                                    .await
                                    .map(|games| {
                                        games.into_iter().take(PER_PAGE).map(game_view).collect_view()
                                    })
                            })
                        }}

                    </Transition>
                </tbody>
            </table>
        </div>
    }
}
