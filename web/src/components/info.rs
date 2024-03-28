use leptos::ev::keydown;
use leptos::*;
use leptos_use::{use_document, use_event_listener};
use web_sys::KeyboardEvent;

#[component]
pub fn ControlsInfoButton(set_show_info: WriteSignal<bool>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="fixed bottom-8 right-8 text-4xl h-12 w-12 rounded-full border border-black bg-white text-gray-900"
            on:click=move |_| set_show_info(true)
        >
            "?"
        </button>
    }
}

#[component]
pub fn ControlsInfoModal(set_show_info: WriteSignal<bool>) -> impl IntoView {
    let key_class = "rounded bg-neutral-600 dark:bg-neutral-900 text-zinc-200 font-light p-1 px-2";
    view! {
        <div
            class="fixed flex flex-col justify-center items-center left-0 right-0 top-0 bottom-0 z-50 bg-neutral-500/50"
            on:click=move |_| set_show_info(false)
        >
            <div
                class="flex flex-col rounded-lg border border-black shadow-lg text-gray-900 dark:text-gray-300 bg-slate-200 dark:bg-slate-800 p-8 w-10/12 max-w-md"
                on:click=move |ev| ev.stop_propagation()
            >
                <h2 class="text-2xl font-bold tracking-wide mb-3">Controls</h2>
                <div class="text-l mb-2">
                    <span class=key_class>"Left Click"</span>
                    " or "
                    <span class=key_class>"Spacebar"</span>
                    " to reveal cell"
                </div>
                <div class="text-l my-2">
                    <span class=key_class>"Right Click"</span>
                    " or "
                    <span class=key_class>"F"</span>
                    " to plant flag"
                </div>
                <div class="text-l my-2">
                    <span class=key_class>"Right + Left Click"</span>
                    " or "
                    <span class=key_class>"D"</span>
                    " to reveal adjacent cells"
                </div>
                <div class="text-l my-2">
                    <span class=key_class>"?"</span>
                    " to open "
                    <span class="font-medium">"Controls"</span>
                    " menu"
                </div>
                <div class="text-l my-2">
                    <span class=key_class>"Esc"</span>
                    " to close "
                    <span class="font-medium">"Controls"</span>
                    " menu"
                </div>
                <div class="text-l my-2">
                    <span class="font-bold">"Note:"</span>
                    " Each player gets one "
                    <span class="font-medium">"\"Super Click\""</span>
                    " - that is, the first hidden cell they reveal that is not adjacent to any revealed cells is guaranteed to be an empty cell"
                </div>
            </div>
        </div>
    }
}

pub fn use_controls_info_keybinds(set_show_info: WriteSignal<bool>) {
    let _ = use_event_listener(use_document(), keydown, move |ev: KeyboardEvent| {
        match ev.key().as_str() {
            "?" => set_show_info(true),
            "Escape" => set_show_info(false),
            _ => {}
        }
    });
}
