mod display_name;
mod game_history;
mod stats;

use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;

use web_auth::FrontendUser;

use super::auth::{LogOutForm, Logout};
use display_name::SetDisplayName;
use game_history::GameHistory;
use stats::{PlayerStatsTable, TimelineStatsGraphs};

#[cfg(feature = "ssr")]
use super::auth::get_user;

#[component]
pub fn ProfileView(
    user: Resource<Option<FrontendUser>, JsonSerdeCodec>,
    logout: ServerAction<Logout>,
    user_updated: WriteSignal<String>,
) -> impl IntoView {
    let title = move || {
        let user = user.get().flatten();
        let is_user = user.is_some();
        let display_name = user.and_then(|u| u.display_name);
        format!(
            "Profile - {}",
            FrontendUser::display_name_or_anon(display_name.as_ref(), is_user)
        )
    };

    let user_profile = move |user: Option<FrontendUser>| {
        match user {
            Some(user) => view! {
                <>
                    <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
                        <SetDisplayName user user_updated />
                        <div class="w-full max-w-xs h-6">
                            <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                                <hr class="w-full" />
                            </span>
                        </div>
                        <LogOutForm logout />
                        <div class="w-full max-w-xs h-6">
                            <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                                <hr class="w-full" />
                            </span>
                        </div>
                        <PlayerStatsTable />
                        <TimelineStatsGraphs />
                        <GameHistory />
                    </div>
                </>
            }.into_any(),
            _ => view! { <Redirect path="/auth/login" /> }.into_any(),
        }
    };

    view! {
        <Title text=title />
        <Suspense fallback=move || ()>{move || { user.get().map(user_profile) }}</Suspense>
    }
}
