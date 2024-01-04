use leptos::*;
use leptos_router::*;

use super::FrontendUser;

#[component]
pub fn Header(user: Option<FrontendUser>) -> impl IntoView {
    view! {
        <header>
            <A href="/">
                <h2>Minesweeper</h2>
            </A>
            {move || match &user {
                None => {
                    view! { <span>"Guest (" <A href="/auth/login">Log in</A> ")"</span> }
                        .into_view()
                }
                Some(user) => {
                    let name = user.display_name_or_anon();
                    view! { <span>{name} " (" <A href="/profile">Profile</A> ")"</span> }
                        .into_view()
                }
            }}

        </header>
    }
}
