use leptos::{ev, prelude::*};
use leptos_use::{use_document, use_event_listener};
use web_sys::KeyboardEvent;

use game_ui::{
    icons::{Mine, Star, Trophy},
    player_icon_holder,
};

#[component]
pub fn ControlsInfoButton(set_show_info: WriteSignal<bool>) -> impl IntoView {
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

#[component]
pub fn ControlsInfoModal(set_show_info: WriteSignal<bool>) -> impl IntoView {
    let key_class = "rounded whitespace-nowrap bg-neutral-600 dark:bg-neutral-900 text-zinc-200 font-light p-1 px-2";
    view! {
        <div
            class="fixed flex flex-col justify-center items-center left-0 right-0 top-0 bottom-0 z-50 bg-neutral-500/50"
            on:click=move |_| set_show_info(false)
        >
            <div
                class="flex flex-col rounded-lg border border-black shadow-lg text-gray-900 dark:text-gray-300 bg-slate-200 dark:bg-slate-800 p-8 w-10/12 max-w-md max-h-10/12 overflow-auto"
                on:click=move |ev| ev.stop_propagation()
            >
                <h2 class="text-2xl font-bold tracking-wide my-3">"Controls"</h2>
                <div class="text-l my-2">
                    <span class=key_class>"Left Click"</span>
                    " or "
                    <span class=key_class>"Spacebar"</span>
                    " or "
                    <span class=key_class>"Tap (touchscreen)"</span>
                    " to reveal cell"
                </div>
                <div class="text-l my-2">
                    <span class=key_class>"Right Click"</span>
                    " or "
                    <span class=key_class>"F"</span>
                    " or "
                    <span class=key_class>"Long Press (touchscreen)"</span>
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
                <h2 class="text-2xl font-bold tracking-wide my-3">"Player Badges"</h2>
                <div class="text-l my-2">
                    <span class=player_icon_holder!("bg-red-600")>
                        <Mine />
                    </span>
                    <span class="font-medium">"Dead"</span>
                    " - the player died by revealing a mine"
                </div>
                <div class="text-l my-2">
                    <span class=player_icon_holder!("bg-green-800")>
                        <Trophy />
                    </span>
                    <span class="font-medium">"Top Score"</span>
                    " - the player had the highest score in a multiplayer minesweeper game"
                </div>
                <div class="text-l my-2">
                    <span class=player_icon_holder!("bg-black")>
                        <Star />
                    </span>
                    <span class="font-medium">"Victory Click"</span>
                    " - the player revealed the final non-mine cell on the map"
                </div>
                <h2 class="text-2xl font-bold tracking-wide my-3">"Multiplayer Rules"</h2>
                <div class="text-l my-2">
                    "Multiple players trying to reveal the same cell is hadled first-click-wins"
                </div>
                <div class="text-l my-2">
                    " Each player gets one " <span class="font-medium">"\"Super Click\""</span>
                    " - that is, the first hidden cell they reveal that is not within a 2-cell distance to any revealed cells is guaranteed to be an empty cell"
                </div>
                <div class="text-l my-2">
                    "If a game is created by a " <span class="font-medium">"Guest"</span>
                    ", then any player can start the game.  If the game is created by a "
                    <span class="font-medium">"Logged In User"</span>
                    ", only the creator can start the game"
                </div>
            </div>
        </div>
    }
}

pub fn use_controls_info_keybinds(set_show_info: WriteSignal<bool>) {
    let _ = use_event_listener(
        use_document(),
        ev::keydown,
        move |ev: KeyboardEvent| match ev.key().as_str() {
            "?" => set_show_info(true),
            "Escape" => set_show_info(false),
            _ => {}
        },
    );
}
