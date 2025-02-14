use leptos::prelude::*;
use leptos_router::components::*;

use game_ui::button_class;

#[component]
pub fn OpenReplayButton() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center space-y-4 mb-8">
            <A
                href="replay"
                attr:class=button_class!(
                    "w-full max-w-xs h-8",
                    "bg-neutral-700 hover:bg-neutral-800/90 text-white"
                )
            >
                "Open Replay"
            </A>
        </div>
    }
}
