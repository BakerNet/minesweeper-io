use leptos::either::*;
use leptos::prelude::*;
use leptos_router::components::*;
use serde::{Deserialize, Serialize};

use super::GameMode;
use crate::{
    components::icons::{IconTooltip, Mine, Star, Trophy},
    player_class, player_icon_holder,
};

#[cfg(feature = "ssr")]
use super::GameSettings;
#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};

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

#[server]
pub async fn get_player_games() -> Result<Vec<PlayerGame>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let user = auth_session.user.ok_or(ServerFnError::new(
        "Cannot find player games when not logged in".to_string(),
    ))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    let games = game_manager
        .get_player_games_for_user(&user)
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
    let player_games = Resource::new(|| (), move |_| async { get_player_games().await });
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
                        attr:class="text-sky-800 hover:text-sky-500 font-medium"
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
    view! {
        <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200">"Game History"</h2>
        <div class="max-w-full overflow-x-auto">
            <table class="border border-solid border-slate-400 border-collapse table-auto text-sm text-center bg-neutral-200/80 dark:bg-neutral-800/80">
                <thead>
                    <tr>
                        <th class=header_class>"Game"</th>
                        <th class=header_class>"Date"</th>
                        <th class=header_class>"Game Mode"</th>
                        <th class=header_class>"Duration"</th>
                        <th class=header_class>"Status"</th>
                        <th class=header_class>"Score"</th>
                    </tr>
                </thead>
                <tbody>
                    <Suspense fallback=move || {
                        (0..5).map(loading_row).collect_view()
                    }>

                        {move || {
                            Suspend::new(async move {
                                player_games
                                    .await
                                    .map(|games| {
                                        games.into_iter().map(game_view).collect_view()
                                    })
                            })
                        }}

                    </Suspense>
                </tbody>
            </table>
        </div>
    }
}
