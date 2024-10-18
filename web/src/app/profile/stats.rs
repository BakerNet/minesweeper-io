use std::collections::VecDeque;

use anyhow::{bail, Result};
use leptos::prelude::*;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
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
    let td_class =
        "border border-slate-100 dark:border-slate-700 py-1 px-2 text-gray-900 dark:text-gray-200";
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

fn parse_stats(stats: &Vec<(bool, i64)>) -> (Vec<(usize, f64)>, Vec<(usize, f64)>) {
    let len = stats.len();
    let (_, _, speed_series, winrate_series) = stats.iter().enumerate().fold(
        (
            VecDeque::with_capacity(10),
            VecDeque::with_capacity(10),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        ),
        |(mut speed_acc, mut wr_acc, mut speed_series, mut wr_series), (i, (v, s))| {
            if wr_acc.len() == 10 {
                wr_acc.pop_front();
            }
            wr_acc.push_back(*v);
            if *v {
                if speed_acc.len() == 10 {
                    speed_acc.pop_front();
                }
                speed_acc.push_back(*s);
            }
            let ave_wr =
                wr_acc.iter().copied().filter(|b| *b).count() as f64 / wr_acc.len() as f64 * 100.0;
            let ave_time =
                speed_acc.iter().map(|x| *x as f64).sum::<f64>() / speed_acc.len() as f64;
            speed_series.push((i, ave_time));
            wr_series.push((i, ave_wr));
            (speed_acc, wr_acc, speed_series, wr_series)
        },
    );
    (speed_series, winrate_series)
}

fn draw_chart(mode: GameMode, stats: &Vec<(bool, i64)>) -> Result<()> {
    let len = stats.len();

    let (speed_series, winrate_series) = parse_stats(stats);

    let max = speed_series
        .iter()
        .map(|(_, y)| *y)
        .max_by(|a, b| a.total_cmp(b))
        .expect("Should be able to find max");
    let canvas = match mode {
        GameMode::ClassicBeginner => "beginner_stats",
        GameMode::ClassicIntermediate => "intermediate_stats",
        GameMode::ClassicExpert => "expert_stats",
        _ => bail!("Mode is not Classic"),
    };

    let backend = CanvasBackend::new(canvas).expect("canvas should exist");
    let root = backend.into_drawing_area();
    let largefont: FontDesc = ("sans-serif", 20.0).into();
    let small_font: FontDesc = ("sans-serif", 14.0).into();
    let tiny_font: FontDesc = ("sans-serif", 12.0).into();

    root.fill(&WHITE)?;

    let root = root.titled(&format!("{} Stats", mode.short_name()), largefont)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(2)
        .caption("10 game moving average", small_font.clone())
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .build_cartesian_2d(0usize..len, 0.0..max + 5.0)?
        .set_secondary_coord(0usize..len - 1, 0.0..100.0);

    let drop_decimal_places = |x: &f64| format!("{:.0}", x);

    chart
        .configure_mesh()
        .max_light_lines(1)
        .disable_x_mesh()
        .x_labels(20.min(len))
        .x_desc("Games")
        .y_labels(10)
        .y_desc("Seconds")
        .y_label_formatter(&drop_decimal_places)
        .y_label_offset(-10)
        .axis_desc_style(small_font.clone())
        .y_label_style(tiny_font.clone())
        .draw()?;

    chart
        .configure_secondary_axes()
        .y_labels(10)
        .y_desc("Winrate")
        .y_label_formatter(&drop_decimal_places)
        .y_label_offset(-10)
        .axis_desc_style(small_font)
        .label_style(tiny_font)
        .draw()?;

    let speed_style = ShapeStyle {
        color: RED.into(),
        filled: false,
        stroke_width: 4,
    };
    chart
        .draw_series(LineSeries::new(speed_series.into_iter(), speed_style).point_size(1))?
        .label("Time")
        .legend(|(x, y)| PathElement::new(vec![(x, y - 5), (x + 20, y - 5)], RED));

    let time_style = ShapeStyle {
        color: BLUE.into(),
        filled: false,
        stroke_width: 4,
    };
    chart
        .draw_secondary_series(
            LineSeries::new(winrate_series.into_iter(), time_style).point_size(1),
        )?
        .label("Winrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y - 5), (x + 20, y - 5)], BLUE));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerLeft)
        .background_style(RGBAColor(128, 128, 128, 0.4))
        .draw()?;

    root.present()?;
    Ok(())
}

#[component]
pub fn TimelineStatsGraphs() -> impl IntoView {
    let timeline_stats = Resource::new(|| (), move |_| async { get_timeline_stats().await });

    Effect::watch(
        move || timeline_stats.get(),
        |tstats, _, _| {
            log::debug!("{:?}", tstats);
            let stats = if let Some(Ok(stats)) = tstats {
                stats
            } else {
                return;
            };
            if let Err(e) = draw_chart(GameMode::ClassicBeginner, &stats.beginner) {
                log::debug!("Unable to draw chart: {}", e);
            };
        },
        true,
    );

    view! {
        <canvas id="beginner_stats" width="500px" height="300px" />
    }
}
