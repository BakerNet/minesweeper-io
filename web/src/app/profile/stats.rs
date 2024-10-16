use std::collections::VecDeque;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::GameMode;

#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
#[cfg(feature = "ssr")]
use crate::models::game::GameStats;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameModeStats {
    pub played: usize,
    pub best_time: usize,
    pub average_time: f64,
    pub victories: usize,
}

#[cfg(feature = "ssr")]
impl From<GameStats> for GameModeStats {
    fn from(value: GameStats) -> Self {
        Self {
            played: value.played as usize,
            best_time: value.best_time as usize,
            average_time: value.average_time,
            victories: value.victories as usize,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub beginner: GameModeStats,
    pub intermediate: GameModeStats,
    pub expert: GameModeStats,
}

#[server]
pub async fn get_player_stats() -> Result<PlayerStats, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let user = auth_session.user.ok_or(ServerFnError::new(
        "Cannot find player games when not logged in".to_string(),
    ))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    let stats = game_manager
        .get_aggregate_stats_for_user(&user)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let beginner = GameModeStats::from(stats.beginner);
    let intermediate = GameModeStats::from(stats.intermediate);
    let expert = GameModeStats::from(stats.expert);

    Ok(PlayerStats {
        beginner,
        intermediate,
        expert,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelinePlayerStats {
    pub beginner: Vec<(bool, i64)>,
    pub intermediate: Vec<(bool, i64)>,
    pub expert: Vec<(bool, i64)>,
}

#[server]
pub async fn get_timeline_stats() -> Result<TimelinePlayerStats, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let user = auth_session.user.ok_or(ServerFnError::new(
        "Cannot find player games when not logged in".to_string(),
    ))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    let stats = game_manager
        .get_timeline_stats_for_user(&user)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let beginner = stats.beginner;
    let intermediate = stats.intermediate;
    let expert = stats.expert;

    Ok(TimelinePlayerStats {
        beginner,
        intermediate,
        expert,
    })
}

#[component]
pub fn PlayerStatsTable() -> impl IntoView {
    let td_class = "border border-slate-100 dark:border-slate-700 py-1 px-2";
    let header_class = "border dark:border-slate-600 font-medium p-4 text-gray-900 dark:text-gray-200 bg-neutral-500/50";
    let player_stats = Resource::new(|| (), move |_| async { get_player_stats().await });

    let mode_view = move |mode: GameMode, stats: GameModeStats| {
        view! {
            <tr>
                <td class=td_class>{mode.short_name()}</td>
                <td class=td_class>{stats.played}</td>
                <td class=td_class>
                    {if stats.played > 0 {
                        format!("{}%", stats.victories * 100 / stats.played)
                    } else {
                        "N/A".to_string()
                    }}
                </td>
                <td class=td_class>
                    {if stats.played > 0 {
                        format!("{}s", stats.best_time)
                    } else {
                        "N/A".to_string()
                    }}
                </td>
                <td class=td_class>
                    {if stats.played > 0 {
                        format!("{:.1}s", stats.average_time)
                    } else {
                        "N/A".to_string()
                    }}
                </td>
            </tr>
        }
    };
    view! {
        <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200">"Stats"</h2>
        <div class="max-w-full overflow-x-auto">

            <table class="border border-solid border-slate-400 border-collapse table-auto text-sm text-center bg-neutral-200/80 dark:bg-neutral-800/80">
                <thead>
                    <tr>
                        <th class=header_class>"Mode"</th>
                        <th class=header_class>"Played"</th>
                        <th class=header_class>"Winrate"</th>
                        <th class=header_class>"Best Time"</th>
                        <th class=header_class>"Average Time"</th>
                    </tr>
                </thead>
                <tbody>
                    <Suspense>
                        {move || Suspend::new(async move {
                            let stats = player_stats.await;
                            stats
                                .map(|stats| {
                                    view! {
                                        <>
                                            {mode_view(GameMode::ClassicBeginner, stats.beginner)}
                                            {mode_view(
                                                GameMode::ClassicIntermediate,
                                                stats.intermediate,
                                            )} {mode_view(GameMode::ClassicExpert, stats.expert)}
                                        </>
                                    }
                                })
                        })}
                    </Suspense>
                </tbody>
            </table>
        </div>
    }
}

#[derive(Clone, Copy, Debug)]
struct GraphPoint {
    games: usize,
    winrate: f64,
    speed: f64,
}

#[component]
pub fn TimelineStatsGraphs() -> impl IntoView {
    let timeline_stats = Resource::new(|| (), move |_| async { get_timeline_stats().await });

    let mode_view = move |mode: GameMode, stats: Vec<(bool, i64)>| {
        let len = stats.len();
        // TODO - when this is > 100, chunk it up
        let (_, _, collections) = stats.into_iter().enumerate().fold(
            (
                VecDeque::with_capacity(10),
                VecDeque::with_capacity(10),
                Vec::with_capacity(len),
            ),
            |(mut speed_acc, mut wr_acc, mut collections), (i, (v, s))| {
                if wr_acc.len() == 10 {
                    wr_acc.pop_front();
                }
                wr_acc.push_back(v);
                if v {
                    if speed_acc.len() == 10 {
                        speed_acc.pop_front();
                    }
                    speed_acc.push_back(s);
                }
                let ave_wr = wr_acc.iter().copied().filter(|b| *b).count() as f64
                    / wr_acc.len() as f64
                    * 100.0;
                let ave_time =
                    speed_acc.iter().map(|x| *x as f64).sum::<f64>() / speed_acc.len() as f64;
                collections.push(GraphPoint {
                    games: i,
                    winrate: ave_wr,
                    speed: ave_time,
                });
                (speed_acc, wr_acc, collections)
            },
        );

        view! {
            <div>{mode.short_name()}": "</div>
            <div>{format!("{:?}", collections)}</div>
        }
    };

    view! {
        <Suspense>
            {move || Suspend::new(async move {
                let stats = timeline_stats.await;
                stats
                    .map(|stats| {
                        view! {
                            <>
                                {mode_view(GameMode::ClassicBeginner, stats.beginner)}
                                {mode_view(GameMode::ClassicIntermediate, stats.intermediate)}
                                {mode_view(GameMode::ClassicExpert, stats.expert)}
                            </>
                        }
                    })
            })}
        </Suspense>
    }
}
