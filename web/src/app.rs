mod auth;
mod error_template;
mod footer;
mod header;
mod home;
mod info;
mod login;
mod minesweeper;
mod profile;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use auth::{get_frontend_user, Login, Logout};
use error_template::{AppError, ErrorTemplate};
use footer::Footer;
use header::Header;
use home::HomeView;
use info::ControlsInfo;
use login::LoginView;
use minesweeper::{ActiveGames, GameView, GameWrapper, RecentGames, ReplayView};
use profile::ProfileView;

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
                <HashedStylesheet options />
                <meta name="color-scheme" content="dark light" />
                <link rel="shortcut icon" type="image/ico" href="/favicon.ico" />
                <script>
                    r#"
                    // On page load or when changing themes, best to add inline in `head` to avoid FOUC
                    if (
                        localStorage.getItem('leptos-use-color-scheme') === 'dark' ||
                        (!('leptos-use-color-scheme' in localStorage) && window.matchMedia('(prefers-color-scheme: dark)').matches)
                    ) {
                        document.documentElement.classList.add('dark')
                        localStorage.setItem('leptos-use-color-scheme', 'dark')
                    } else {
                        document.documentElement.classList.remove('dark')
                        localStorage.setItem('leptos-use-color-scheme', 'light')
                    }
                    "#
                </script>
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let login = ServerAction::<Login>::new();
    let logout = ServerAction::<Logout>::new();
    let (user_update, user_updated) = signal("".to_string());

    let user = Resource::new(
        move || (login.version().get(), logout.version().get(), user_update()),
        move |_| async { get_frontend_user().await.ok().flatten() },
    );

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Title formatter=|title| format!("Minesweeper - {title}") />
        <Router>
            <main class="flex flex-col min-h-screen bg-white dark:bg-gray-900">
                <Header user />
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! { <ErrorTemplate outside_errors /> }.into_view()
                }>
                    <Route path=path!("/") view=HomeView />
                    <Route path=path!("/auth/login") view=move || view! { <LoginView login /> } />
                    <Route
                        path=path!("/profile")
                        view=move || {
                            view! { <ProfileView user logout user_updated /> }
                        }
                    />
                    <ParentRoute path=path!("/game/:id") view=GameWrapper>
                        <Route path=path!("/replay") view=ReplayView />
                        <Route path=path!("/") view=GameView />
                    </ParentRoute>
                    <Route path=path!("/active") view=ActiveGames />
                    <Route path=path!("/recent") view=RecentGames />
                </Routes>
                <Footer />
                <ControlsInfo />
            </main>
        </Router>
    }
}
