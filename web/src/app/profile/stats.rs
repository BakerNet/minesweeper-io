use anyhow::Result;
use leptos::prelude::*;

use game_ui::{
    GameMode, PlayerStats, PlayerStatsRow, PlayerStatsTable, TimelineStats, TimelineStatsGraphs,
};

#[cfg(feature = "ssr")]
use game_manager::{models::GameStats, GameManager};
#[cfg(feature = "ssr")]
use game_ui::{parse_timeline_stats, PlayerGameModeStats};
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

#[cfg(feature = "ssr")]
fn map_player_game_stats(value: GameStats) -> PlayerGameModeStats {
    PlayerGameModeStats {
        played: value.played as usize,
        best_time: value.best_time as usize,
        average_time: value.average_time,
        victories: value.victories as usize,
    }
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

    let beginner = map_player_game_stats(stats.beginner);
    let intermediate = map_player_game_stats(stats.intermediate);
    let expert = map_player_game_stats(stats.expert);

    Ok(PlayerStats {
        beginner,
        intermediate,
        expert,
    })
}

#[server]
pub async fn get_timeline_stats() -> Result<TimelineStats, ServerFnError> {
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

    Ok(TimelineStats {
        beginner: parse_timeline_stats(&stats.beginner),
        intermediate: parse_timeline_stats(&stats.intermediate),
        expert: parse_timeline_stats(&stats.expert),
    })
}

#[component]
pub fn WebPlayerStatsTable() -> impl IntoView {
    let player_stats = Resource::new(|| (), move |_| async { get_player_stats().await });
    view! {
        <PlayerStatsTable>
            <Suspense>
                {move || Suspend::new(async move {
                    let stats = player_stats.await;
                    stats
                        .map(|stats| {
                            view! {
                                <>
                                    <PlayerStatsRow
                                        mode=GameMode::ClassicBeginner
                                        stats=stats.beginner
                                    />
                                    <PlayerStatsRow
                                        mode=GameMode::ClassicIntermediate
                                        stats=stats.intermediate
                                    />
                                    <PlayerStatsRow
                                        mode=GameMode::ClassicExpert
                                        stats=stats.expert
                                    />
                                </>
                            }
                        })
                })}
            </Suspense>
        </PlayerStatsTable>
    }
}

#[component]
pub fn WebTimelineStatsGraphs() -> impl IntoView {
    let timeline_stats = Resource::new(
        || (),
        move |_| async {
            let stats = get_timeline_stats().await;
            stats.ok()
        },
    );
    let timeline_stats = Signal::derive(move || timeline_stats.get().flatten());

    view! { <TimelineStatsGraphs timeline_stats /> }
}
