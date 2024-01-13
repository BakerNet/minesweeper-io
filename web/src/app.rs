pub mod auth;
mod error_template;
mod header;
mod home;
mod login;
pub mod minesweeper;
mod profile;

use auth::*;
use error_template::{AppError, ErrorTemplate};
use header::Header;
use home::HomePage;
use login::LoginPage;
use minesweeper::{players::Players, Game};
use profile::Profile;

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use crate::app::minesweeper::players::ShowPlayers;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendUser {
    pub display_name: Option<String>,
}

impl FrontendUser {
    pub fn display_name_or_anon(display_name: &Option<String>) -> String {
        if let Some(name) = display_name {
            name.to_owned()
        } else {
            "Anonymous".to_string()
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let login = create_server_action::<LogIn>();
    let logout = create_server_action::<LogOut>();
    let (user_update, user_updated) = create_signal("".to_string());

    let user = create_resource(
        move || (login.version().get(), logout.version().get(), user_update()),
        move |_| async { get_frontend_user().await.ok().flatten() },
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/minesweeper-web.css"/>

        // sets the document title
        <Title text="Welcome to Minesweeper"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main class="flex flex-col min-h-screen bg-white dark:bg-gray-900">
                <Header user/>
                <Routes>
                    <Route path="/" view=move || view! { <HomePage user/> }/>
                    <Route path="/auth/login" view=move || view! { <LoginPage login/> }/>
                    <Route
                        path="/profile"
                        view=move || {
                            view! {
                                <Transition fallback=move || {
                                    view! { <span>"Loading..."</span> }
                                }>
                                    {move || {
                                        if let Some(Some(user)) = user.get() {
                                            view! { <Profile user logout user_updated/> }
                                        } else {
                                            let mut outside_errors = Errors::default();
                                            outside_errors
                                                .insert_with_default_key(AppError::NotLoggedIn);
                                            view! { <ErrorTemplate outside_errors/> }
                                        }
                                    }}

                                </Transition>
                            }
                        }
                    />

                    <Route path="/game/:id" view=|| view! { <Game/> }>
                        <Route path="players" view=|| view! { <Players/> }/>
                        <Route path="" view=|| view!{ <ShowPlayers /> } />

                    </Route>
                </Routes>
            </main>
        </Router>
    }
}
