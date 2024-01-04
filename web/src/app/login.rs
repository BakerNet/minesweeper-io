use leptos::*;

use super::auth::{LogIn, Login, OAuthTarget};

#[component]
pub fn LoginPage(login: Action<LogIn, Result<String, ServerFnError>>) -> impl IntoView {
    view! {
        <>
            <Login login target=OAuthTarget::GOOGLE />
            <Login login target=OAuthTarget::REDDIT />
            <Login login target=OAuthTarget::GITHUB />
        </>
    }
}
