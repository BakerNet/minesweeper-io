use leptos::*;

use super::{minesweeper::JoinOrCreateGame, FrontendUser};

/// Renders the home page of your application.
#[component]
pub fn HomePage<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    view! {
        <h1>"Welcome to Minesweeper!"</h1>
        <JoinOrCreateGame user/>
    }
}
