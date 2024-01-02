use crate::auth::*;
use crate::error_template::{AppError, ErrorTemplate};
use crate::game::players::Players;
use crate::game::Game;
use crate::models::user::User;
use crate::views::home::HomePage;
use crate::views::login::LoginPage;
use crate::views::profile::Profile;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use cfg_if::cfg_if;

cfg_if! { if #[cfg(feature = "ssr")] {
    use leptos::LeptosOptions;
    use axum::extract::FromRef;
    use leptos_router::RouteListing;
    /// This takes advantage of Axum's SubStates feature by deriving FromRef. This is the only way to have more than one
    /// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
    #[derive(FromRef, Debug, Clone)]
    pub struct AppState{
        pub leptos_options: LeptosOptions,
        pub routes: Vec<RouteListing>,
    }
}}

#[component]
pub fn App() -> impl IntoView {
    let login = create_server_action::<LogIn>();
    let logout = create_server_action::<LogOut>();
    let (user_update, user_updated) = create_signal("".to_string());

    let user = create_resource(
        move || (login.version().get(), logout.version().get(), user_update()),
        move |_| get_user(),
    );
    let user_into = move || {
        let user = user.get();
        if let Some(res) = user {
            res.ok()
        } else {
            Some(None)
        }
    };

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/minesweeper-web.css"/>
        <Stylesheet id="leptos" href="/pkg/minesweeper-web.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Transition fallback=move || {
                    view! { <Header user=None/> }
                }>
                    {move || {
                        let user = user_into();
                        user.map(|user| view! { <Header user/> })
                    }}

                </Transition>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/auth/login" view=move || view! { <LoginPage login/> }/>
                    <Route
                        path="/profile"
                        view=move || {
                            view! {
                                <Transition fallback=move || {
                                    view! { <span>"Loading..."</span> }
                                }>
                                    {move || {
                                        if let Some(Ok(Some(user))) = user.get() {
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
                    <Route path="/game/:id" view=|| view! { <Game rows=50 cols=50/> }>
                        <Route path="players" view=|| view! { <Players/> }/>
                        <Route
                            path=""
                            view=|| {
                                view! { <A href="players">"Join Game / Scoreboard"</A> }
                            }
                        />
                    </Route>

                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Header(user: Option<User>) -> impl IntoView {
    view! {
        <header>
            <A href="/">
                <h1>Minesweeper</h1>
            </A>
            {move || match &user {
                None => {
                    view! { <span>"Guest (" <A href="/auth/login">Log in</A> ")"</span> }
                        .into_view()
                }
                Some(user) => {
                    let name = if let Some(name) = &user.display_name {
                        name.to_owned()
                    } else {
                        "Anonymous".to_string()
                    };
                    view! { <span>{name} " (" <A href="/profile">Profile</A> ")"</span> }
                        .into_view()
                }
            }}

        </header>
    }
}
