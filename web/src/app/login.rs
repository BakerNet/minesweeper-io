use leptos::*;

use super::auth::{LogIn, LoginForm, OAuthTarget};

#[component]
pub fn LoginView(login: Action<LogIn, Result<String, ServerFnError>>) -> impl IntoView {
    view! {
        <>
            <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
                <LoginForm login target=OAuthTarget::Google />
                <LoginForm login target=OAuthTarget::Reddit />
                <LoginForm login target=OAuthTarget::Github />
            </div>
        </>
    }
}
