use leptos::*;

use crate::game::StartGame;

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to Minesweeper!"</h1>
        <StartGame />
    }
}
