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
    pub fn display_name_or_anon(display_name: &Option<String>, is_user: bool) -> String {
        if let Some(name) = display_name {
            name.to_owned()
        } else if is_user {
            "Anonymous".to_string()
        } else {
            "Guest".to_string()
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
    provide_context(user);

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Script>
            r#"
            // On page load or when changing themes, best to add inline in `head` to avoid FOUC
            if (
                localStorage.getItem("leptos-use-color-scheme") === 'dark' ||
                (!('leptos-use-color-scheme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)
            ) {
                document.documentElement.classList.add('dark')
            } else {
                document.documentElement.classList.remove('dark')
            }
            "#
        </Script>
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
                    <Route path="/" view=move || view! { <HomePage/> }/>
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
                        <Route path="" view=|| view! { <ShowPlayers/> }/>

                    </Route>
                </Routes>
            </main>
        </Router>
    }
}
