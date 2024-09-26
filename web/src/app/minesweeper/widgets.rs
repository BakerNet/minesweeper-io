use leptos::*;
use leptos_use::{
    use_clipboard, use_interval_fn_with_options, use_timeout_fn, utils::Pausable,
    UseClipboardReturn, UseIntervalFnOptions, UseTimeoutFnReturn,
};

use crate::{
    components::icons::{Copy, IconTooltip, Mine, StopWatch},
    widget_icon_holder,
};

#[component]
pub fn GameWidgets(children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center">
            <div class="flex justify-between w-full max-w-xs mb-2">{children()}</div>
        </div>
    }
}

#[component]
pub fn ActiveTimer(
    sync_time: ReadSignal<Option<usize>>,
    completed: ReadSignal<bool>,
) -> impl IntoView {
    let (start_time, set_start_time) = create_signal::<Option<f64>>(None);
    let (display_time, set_display_time) = create_signal::<usize>(0);

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

    Effect::new(move |prev| {
        log::debug!("Timer effect");
        let completed = completed.get();
        let sync_time = sync_time.get();
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
    });

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
pub fn CopyGameLink(game_id: String) -> impl IntoView {
    let (show_tooltip, set_show_tooltip) = create_signal(false);
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
            "show-tooltip"
        } else {
            ""
        }
    };
    #[cfg(not(feature = "ssr"))]
    let origin = { window().location().origin().unwrap_or_default() };
    #[cfg(feature = "ssr")]
    let origin = String::new();
    let url = format!("{}/game/{}", origin, game_id);
    view! {
        <div class="flex flex-col items-center justify-center border-2 rounded-full border-slate-400 bg-neutral-200 text-neutral-800 font-medium px-2">
            <button
                class=copy_class
                on:click=move |_| {
                    copy(&url);
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
