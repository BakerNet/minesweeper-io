use leptos::prelude::*;

use game_ui::info::{use_controls_info_keybinds, ControlsInfoModal};

#[component]
pub fn ControlsInfo() -> impl IntoView {
    let (show_info, set_show_info) = signal(false);
    use_controls_info_keybinds(set_show_info);

    view! {
        <ControlsInfoButton set_show_info />
        <Show when=show_info>
            <ControlsInfoModal set_show_info include_multiplayer=true />
        </Show>
    }
}

#[component]
fn ControlsInfoButton(set_show_info: WriteSignal<bool>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="fixed bottom-2 left-2 sm:right-8 text-4xl h-12 w-12 rounded-full border border-black bg-white text-gray-900"
            on:click=move |_| set_show_info(true)
        >
            "?"
        </button>
    }
}
