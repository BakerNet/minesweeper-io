use leptos::prelude::*;
use leptos_meta::*;

use super::auth::{Login, LoginForm, OAuthTarget};

#[component]
pub fn LoginView(login: ServerAction<Login>) -> impl IntoView {
    view! {
        <Title text="Log In" />
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4 mx-auto w-full max-w-sm">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200 font-bold">"Log In"</h1>
            <div class="text-center pb-8 text-gray-900 dark:text-gray-100">
                "Log in if you want to set your display name, keep game history, or see your game stats & trends"
            </div>
            <LoginForm login target=OAuthTarget::Google />
            <LoginForm login target=OAuthTarget::Reddit />
            <LoginForm login target=OAuthTarget::Github />
            <div class="text-center pt-8 text-gray-900 dark:text-gray-100">
                "Note: None of your personal info is checked or stored - only your username is used to identify your account"
            </div>
        </div>
    }
}
