use leptos::*;

use super::{minesweeper::JoinOrCreateGame, FrontendUser};

/// Renders the home page of your application.
#[component]
pub fn HomePage<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    view! {
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4">
            <h1 class="text-4xl font-bold text-center text-gray-900 dark:text-gray-100 mb-8">
                "Welcome to Minesweeper!"
            </h1>
            <JoinOrCreateGame user/>
        </div>
    }
}
