use leptos::prelude::*;
use leptos_router::components::*;

use game_ui::button_class;

#[component]
pub fn OpenReplayButton() -> impl IntoView {
    view! {
        <A
            href="replay"
            attr:class=button_class!(
                "rounded",
                "bg-blue-600 hover:bg-blue-700/90 text-white"
            )
        >
            "Open Replay"
        </A>
    }
}
