use chrono::DateTime;
use leptos::either::EitherOf4;
use leptos::prelude::*;
use leptos_use::{
    use_clipboard, use_interval_fn_with_options, use_timeout_fn, utils::Pausable,
    UseClipboardReturn, UseIntervalFnOptions, UseTimeoutFnReturn,
};

use crate::{
    icons::{Copy, IconTooltip, Mine, PlayArrow, QuestionMark, Star, StopWatch},
    widget_icon_holder, widget_icon_standalone,
};

#[component]
pub fn GameWidgets(children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center">
            <div class="flex justify-between w-full max-w-xs mb-2">{children()}</div>
        </div>
    }
}

pub fn game_time_from_start_end<T: chrono::TimeZone>(
    start_time: Option<DateTime<T>>,
    end_time: Option<DateTime<T>>,
) -> usize {
    (match (start_time, end_time) {
        (Some(st), Some(et)) => et.signed_duration_since(st).num_seconds(),
        _ => 999,
    }) as usize
}

#[component]
pub fn ActiveTimer(
    sync_time: ReadSignal<Option<usize>>,
    completed: ReadSignal<bool>,
) -> impl IntoView {
    let (start_time, set_start_time) = signal::<Option<f64>>(None);
    let (display_time, set_display_time) = signal::<usize>(0);

    let Pausable {
        is_active,
        pause,
        resume,
    } = use_interval_fn_with_options(
        move || {
            if let Some(st) = start_time.get() {
                if let Some(p) = window().performance() {
                    let base = sync_time.get().unwrap_or(0);
                    let time_since_sync = (p.now() - st).floor() as usize / 1000;
                    let display_time = 999.min(base + time_since_sync);
                    set_display_time(display_time);
                };
            }
        },
        100,
        UseIntervalFnOptions {
            immediate: false,
            immediate_callback: false,
        },
    );

    Effect::watch(
        move || (completed.get(), sync_time.get()),
        move |curr, _, prev| {
            log::debug!("Timer effect");
            let completed = curr.0;
            let sync_time = curr.1;
            if sync_time.is_some() && sync_time != prev.flatten() {
                if let Some(st) = sync_time {
                    set_display_time(st);
                    if let Some(p) = window().performance() {
                        set_start_time(Some(p.now()));
                    };
                };
            }
            if !is_active.get_untracked() && !completed && sync_time.is_some() {
                resume();
            } else if completed {
                pause();
            }
            sync_time
        },
        true,
    );

    view! {
        <div class="flex items-center">
            <span class=widget_icon_holder!("bg-neutral-200")>
                <StopWatch />
            </span>
            <div class="flex flex-col items-center justify-center border-4 border-slate-400 bg-neutral-200 text-neutral-800 text-lg font-bold px-2">
                {display_time}
            </div>
        </div>
    }
}

#[component]
pub fn InactiveMines(num_mines: usize) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <div class="flex flex-col items-center justify-center border-4 border-slate-400 bg-neutral-200 text-neutral-800 text-lg font-bold px-2">
                {num_mines}
            </div>
            <span class=widget_icon_holder!("bg-neutral-200")>
                <Mine />
            </span>
        </div>
    }
}

#[component]
pub fn ActiveMines(num_mines: usize, flag_count: ReadSignal<usize>) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <div class="flex flex-col items-center justify-center border-4 border-slate-400 bg-neutral-200 text-neutral-800 text-lg font-bold px-2">
                {move || num_mines as isize - flag_count.get() as isize}
            </div>
            <span class=widget_icon_holder!("bg-neutral-200")>
                <Mine />
            </span>
        </div>
    }
}

#[component]
pub fn InactiveTimer(game_time: usize) -> impl IntoView {
    let game_time = 999.min(game_time);

    view! {
        <div class="flex items-center">
            <span class=widget_icon_holder!("bg-neutral-200")>
                <StopWatch />
            </span>
            <div class="flex flex-col items-center justify-center border-4 border-slate-400 bg-neutral-200 text-neutral-800 text-lg font-bold px-2">
                {game_time}
            </div>
        </div>
    }
}

#[component]
pub fn CopyGameLink(game_url: String) -> impl IntoView {
    let (show_tooltip, set_show_tooltip) = signal(false);
    let UseClipboardReturn { copy, .. } = use_clipboard();
    let UseTimeoutFnReturn { start, .. } = use_timeout_fn(
        move |_| {
            set_show_tooltip(false);
        },
        1000.0,
    );
    let copy_class = move || {
        let show_tooltip = show_tooltip.get();
        if show_tooltip {
            "show-tooltip cursor-default"
        } else {
            "cursor-pointer"
        }
    };
    view! {
        <div class="flex flex-col items-center justify-center border-2 rounded-full border-slate-400 bg-neutral-200 text-neutral-800 font-medium px-2">
            <button
                class=copy_class
                on:click=move |_| {
                    copy(&game_url);
                    set_show_tooltip(true);
                    start(());
                }
            >
                <span>Copy Link</span>
                <span class=widget_icon_holder!("", true)>
                    <Copy />
                    <IconTooltip>Copied</IconTooltip>
                </span>
            </button>
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    NotStarted,
    Active,
    Victory,
    Dead,
}

#[component]
pub fn GameStateWidget(
    completed: ReadSignal<bool>,
    victory: ReadSignal<bool>,
    dead: ReadSignal<bool>,
    sync_time: ReadSignal<Option<usize>>,
) -> impl IntoView {
    let game_state = move || {
        let is_completed = completed.get();
        let is_victory = victory.get();
        let is_dead = dead.get();
        let has_started = sync_time.get().is_some();

        if is_completed {
            if is_victory {
                GameState::Victory
            } else if is_dead {
                GameState::Dead
            } else {
                GameState::Active // fallback
            }
        } else if has_started {
            GameState::Active
        } else {
            GameState::NotStarted
        }
    };

    view! {
        <div class="flex items-center">
            <div class="flex justify-center border-4 border-slate-400 bg-neutral-200 text-neutral-800 text-lg font-bold">
                {move || {
                    match game_state() {
                        GameState::NotStarted => EitherOf4::A(view! {
                            <span class=widget_icon_standalone!("bg-gray-400")>
                                <QuestionMark />
                            </span>
                        }),
                        GameState::Active => EitherOf4::B(view! {
                            <span class=widget_icon_standalone!("bg-blue-500")>
                                <PlayArrow />
                            </span>
                        }),
                        GameState::Victory => EitherOf4::C(view! {
                            <span class=widget_icon_standalone!("bg-black", true)>
                                <Star />
                                <IconTooltip>"Victory"</IconTooltip>
                            </span>
                        }),
                        GameState::Dead => EitherOf4::D(view! {
                            <span class=widget_icon_standalone!("bg-red-600", true)>
                                <Mine />
                                <IconTooltip>"Dead"</IconTooltip>
                            </span>
                        }),
                    }
                }}
            </div>
        </div>
    }
}
