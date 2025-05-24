use std::collections::VecDeque;

use anyhow::Result;
use codee::string::JsonSerdeWasmCodec;
use full_palette::{CYAN_200, CYAN_500, INDIGO_200, WHITE};
use leptos::{html::Canvas, prelude::*};
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;

use crate::{button_class, GameMode};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerGameModeStats {
    pub played: usize,
    pub best_time: usize,
    pub average_time: f64,
    pub victories: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub beginner: PlayerGameModeStats,
    pub intermediate: PlayerGameModeStats,
    pub expert: PlayerGameModeStats,
}

#[component]
pub fn PlayerStatsRow(mode: GameMode, stats: PlayerGameModeStats) -> impl IntoView {
    let td_class =
        "border border-slate-100 dark:border-slate-700 py-1 px-2 text-gray-900 dark:text-gray-200";
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
                {if stats.played > 0 { format!("{}s", stats.best_time) } else { "N/A".to_string() }}
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
}

#[component]
pub fn PlayerStatsTable(children: Children) -> impl IntoView {
    let header_class = "border dark:border-slate-600 font-medium p-4 text-gray-900 dark:text-gray-200 bg-neutral-500/50";

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
                <tbody>{children()}</tbody>
            </table>
        </div>
    }
}

#[component]
fn StatSelectButtons(
    selected: Signal<GameMode>,
    set_selected: WriteSignal<GameMode>,
) -> impl IntoView {
    let classic_modes = [
        GameMode::ClassicBeginner,
        GameMode::ClassicIntermediate,
        GameMode::ClassicExpert,
    ];

    let class_signal = move |mode: GameMode| {
        let selected = selected.get();
        if mode == selected {
            button_class!(
                "w-full rounded-lg",
                "bg-neutral-800 text-neutral-50 border-neutral-500"
            )
        } else {
            button_class!("w-full rounded-lg")
        }
    };

    let mode_button = move |mode: GameMode| {
        view! {
            <div class="flex-1">
                <button
                    type="button"
                    class=move || class_signal(mode)
                    on:click=move |_| {
                        set_selected(mode);
                    }
                >

                    {mode.short_name()}
                </button>
            </div>
        }
    };

    view! {
        <div class="w-full space-y-2">
            <div class="flex w-full space-x-2">{classic_modes.map(mode_button).collect_view()}</div>
        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineGameModeStats {
    pub speed: Vec<(usize, f64)>,
    pub winrate: Vec<(usize, f64)>,
    pub best_time: Vec<(usize, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineStats {
    pub beginner: TimelineGameModeStats,
    pub intermediate: TimelineGameModeStats,
    pub expert: TimelineGameModeStats,
}

pub fn parse_timeline_stats(stats: &[(bool, i64)]) -> TimelineGameModeStats {
    let len = stats.len();
    let (_, _, speed_series, winrate_series, mut best_time_series) = stats.iter().enumerate().fold(
        (
            VecDeque::with_capacity(10),
            VecDeque::with_capacity(10),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        ),
        |(mut speed_acc, mut wr_acc, mut speed_series, mut wr_series, mut best_time),
         (i, (v, s))| {
            let x = i + 1;
            if wr_acc.len() == 10 {
                wr_acc.pop_front();
            }
            wr_acc.push_back(*v);
            let ave_wr =
                wr_acc.iter().copied().filter(|b| *b).count() as f64 / wr_acc.len() as f64 * 100.0;
            wr_series.push((x, ave_wr));

            if *v {
                if speed_acc.len() == 10 {
                    speed_acc.pop_front();
                }
                let s = *s as f64;
                speed_acc.push_back(s);
                let ave_time = speed_acc.iter().sum::<f64>() / speed_acc.len() as f64;
                speed_series.push((x, ave_time));

                if best_time.is_empty() {
                    best_time.push((x, s));
                }
                let prev_best = best_time.last().unwrap().1;
                if s < prev_best {
                    best_time.push((x, prev_best));
                    best_time.push((x, s));
                }
            }

            (speed_acc, wr_acc, speed_series, wr_series, best_time)
        },
    );
    if let Some(best_time_last) = best_time_series.last() {
        if best_time_last.0 < winrate_series.len() {
            best_time_series.push((winrate_series.len(), best_time_last.1));
        }
    }
    TimelineGameModeStats {
        speed: speed_series,
        winrate: winrate_series,
        best_time: best_time_series,
    }
}

fn draw_chart(
    canvas: HtmlCanvasElement,
    mode: GameMode,
    stats: &TimelineGameModeStats,
) -> Result<()> {
    let len = 10.max(stats.winrate.len());

    let TimelineGameModeStats {
        speed: speed_series,
        winrate: winrate_series,
        best_time: best_time_series,
    } = stats;

    let max = speed_series
        .iter()
        .map(|(_, y)| *y)
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or_default();
    let max = 10.0_f64.max(max);

    let backend = CanvasBackend::with_canvas_object(canvas).expect("should be able to init canvas");
    let root = backend.into_drawing_area();
    root.fill(&RGBColor(17, 24, 39))?;

    let large_font = ("sans-serif", 20.0)
        .with_color(WHITE)
        .into_text_style(&root);
    let small_font = ("sans-serif", 14.0)
        .with_color(WHITE)
        .into_text_style(&root);
    let tiny_font = ("sans-serif", 12.0)
        .with_color(WHITE)
        .into_text_style(&root);

    let root = root.titled(&format!("{} Stats", mode.short_name()), large_font)?;

    let mut chart = ChartBuilder::on(&root)
        .margin(2)
        .caption(
            "Winrate & Time - 10 game moving average",
            small_font.clone(),
        )
        .x_label_area_size(35)
        .y_label_area_size(45)
        .right_y_label_area_size(45)
        .build_cartesian_2d(1usize..len, 0.0..max + 5.0)?
        .set_secondary_coord(1usize..len, 0.0..100.0);

    let drop_decimal_places = |x: &f64| format!("{x:.0}");

    chart
        .configure_mesh()
        .max_light_lines(1)
        .disable_x_mesh()
        .x_labels(20.min(len))
        .x_desc("Games")
        .y_labels(10)
        .y_desc("Time (Seconds - victories only)")
        .y_label_formatter(&drop_decimal_places)
        .y_label_offset(-10)
        .axis_desc_style(small_font.clone())
        .y_label_style(tiny_font.clone())
        .x_label_style(tiny_font.clone())
        .bold_line_style(TRANSPARENT)
        .light_line_style(WHITE)
        .axis_style(WHITE)
        .draw()?;

    chart
        .configure_secondary_axes()
        .y_labels(10)
        .y_desc("Winrate (%)")
        .y_label_formatter(&drop_decimal_places)
        .y_label_offset(-10)
        .axis_desc_style(small_font)
        .label_style(tiny_font.clone())
        .axis_style(WHITE)
        .draw()?;

    chart
        .draw_series(
            LineSeries::new(speed_series.clone(), CYAN_200.stroke_width(2).filled()).point_size(2),
        )?
        .label("Time")
        .legend(|(x, y)| PathElement::new(vec![(x, y - 5), (x + 20, y - 5)], CYAN_200));

    chart
        .draw_series(LineSeries::new(
            best_time_series.clone(),
            CYAN_500.stroke_width(2).filled(),
        ))?
        .label("Best Time")
        .legend(|(x, y)| PathElement::new(vec![(x, y - 5), (x + 20, y - 5)], CYAN_500));

    let mut seen_t = 999.0_f64;
    chart.draw_series(PointSeries::of_element(
        best_time_series
            .iter()
            .filter(|(_, t)| {
                let ret = *t != seen_t;
                seen_t = *t;
                ret
            })
            .cloned(),
        2,
        CYAN_500,
        &|coord, size, style| {
            EmptyElement::at(coord)
                + Circle::new((0, 0), size, style)
                + Text::new(format!("{:.0}s", coord.1), (0, 5), tiny_font.clone())
        },
    ))?;

    chart
        .draw_secondary_series(
            LineSeries::new(winrate_series.clone(), INDIGO_200.stroke_width(2).filled())
                .point_size(2),
        )?
        .label("Winrate")
        .legend(|(x, y)| PathElement::new(vec![(x, y - 5), (x + 20, y - 5)], INDIGO_200));

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::LowerLeft)
        .background_style(RGBAColor(65, 65, 65, 0.8))
        .label_font(tiny_font)
        .margin(2)
        .legend_area_size(20)
        .draw()?;

    root.present()?;
    Ok(())
}

#[component]
pub fn TimelineStatsGraphs(timeline_stats: Signal<Option<TimelineStats>>) -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();

    let storage_options = UseStorageOptions::<GameMode, serde_json::Error, JsValue>::default()
        .initial_value(GameMode::ClassicBeginner)
        .delay_during_hydration(true);
    let (selected_mode, set_selected_mode, _) = use_local_storage_with_options::<
        GameMode,
        JsonSerdeWasmCodec,
    >("game_mode_stats", storage_options);

    Effect::watch(
        move || (timeline_stats.get(), selected_mode.get(), canvas_ref.get()),
        |(tstats, mode, canvas), _, _| {
            let canvas: HtmlCanvasElement = if let Some(el) = canvas {
                el.to_owned()
            } else {
                log::debug!("canvas not ready");
                return;
            };
            log::debug!("Stats: {tstats:?}");
            let stats = if let Some(stats) = tstats {
                stats
            } else {
                return;
            };
            let stats = match mode {
                GameMode::ClassicBeginner => &stats.beginner,
                GameMode::ClassicIntermediate => &stats.intermediate,
                GameMode::ClassicExpert => &stats.expert,
                _ => return,
            };
            if let Err(e) = draw_chart(canvas, *mode, stats) {
                log::debug!("Unable to draw chart: {e}");
            };
        },
        true,
    );

    view! {
        <div class="flex flex-col w-full max-w-xs space-y-2">
            <StatSelectButtons selected=selected_mode set_selected=set_selected_mode />
        </div>
        <canvas
            node_ref=canvas_ref
            id="stats_canvas"
            class="max-w-full"
            width="800px"
            height="500px"
        />
    }
}
