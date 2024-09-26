use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::info::{use_controls_info_keybinds, ControlsInfoButton, ControlsInfoModal};

use super::{
    auth::{get_frontend_user, LogIn, LogOut},
    error_template::{AppError, ErrorTemplate},
    header::Header,
    home::HomeView,
    login::LoginView,
    minesweeper::{GameView, GameWrapper, ReplayView},
    profile::ProfileView,
};

#[component]
pub fn App() -> impl IntoView {
    let login = create_server_action::<LogIn>();
    let logout = create_server_action::<LogOut>();
    let (user_update, user_updated) = create_signal("".to_string());
    let (show_info, set_show_info) = create_signal(false);
    use_controls_info_keybinds(set_show_info);

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
        <Stylesheet id="leptos" href="/pkg/minesweeper-web.css" />

        // sets the document title
        <Title text="Welcome to Minesweeper" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <main class="flex flex-col min-h-screen bg-white dark:bg-gray-900">
                <Header user />
                <Routes>
                    <Route path="/" view=HomeView />
                    <Route path="/auth/login" view=move || view! { <LoginView login /> } />
                    <Route
                        path="/profile"
                        view=move || {
                            view! { <ProfileView user logout user_updated /> }
                        }
                    />
                    <Route path="/game/:id" view=GameWrapper>
                        <Route path="/replay" view=ReplayView />
                        <Route path="/" view=GameView />
                    </Route>
                </Routes>
                <ControlsInfoButton set_show_info />
                <Show when=show_info>
                    <ControlsInfoModal set_show_info />
                </Show>
            </main>
        </Router>
    }
}
