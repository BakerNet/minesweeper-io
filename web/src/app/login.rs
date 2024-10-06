use leptos::prelude::*;
use leptos_meta::*;

use super::auth::{Login, LoginForm, OAuthTarget};

#[component]
pub fn LoginView(login: ServerAction<Login>) -> impl IntoView {
    view! {
        <Title text="Log In" />
        <>
            <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
                <LoginForm login target=OAuthTarget::Google />
                <LoginForm login target=OAuthTarget::Reddit />
                <LoginForm login target=OAuthTarget::Github />
            </div>
        </>
    }
}
