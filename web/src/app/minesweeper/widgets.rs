use leptos::*;
use leptos_use::{use_interval_fn_with_options, use_window, UseIntervalFnOptions};

#[component]
pub fn GameWidgets(children: Children) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center">
            <div class="flex justify-between w-full max-w-xs select-none mb-2">
                {children()}
            </div>
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

    let performance = move || {
        let window = use_window();
        let window = window.as_ref();
        window.map(|w| w.performance()).flatten()
    };

    let interval = use_interval_fn_with_options(
        move || {
            if let Some(st) = start_time.get() {
                performance().map(|p| {
                    let base = sync_time.get().unwrap_or(0);
                    let time_since_sync = (p.now() - st).floor() as usize / 1000;
                    set_display_time(base + time_since_sync);
                });
            }
        },
        100,
        UseIntervalFnOptions {
            immediate: false,
            immediate_callback: false,
        },
    );

    create_effect(move |prev: Option<Option<usize>>| {
        let completed = completed.get();
        let sync_time = sync_time.get();
        if sync_time.is_some() && sync_time != prev.flatten() {
            sync_time.map(|st| {
                set_display_time(st);
                performance().map(|p| {
                    set_start_time(Some(p.now()));
                });
            });
        }
        if !(interval.is_active)() && !completed && sync_time.is_some() {
            (interval.resume)();
        } else if completed {
            (interval.pause)();
        }
        sync_time
    });

    view! {<div class="w-16 h16 border border-8 border-slate-400 bg-neutral-200 text-neutral-800 font-bold">{{display_time}}</div>}
}
