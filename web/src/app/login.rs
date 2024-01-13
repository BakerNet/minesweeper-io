use leptos::*;

use super::auth::{LogIn, Login, OAuthTarget};

#[component]
pub fn LoginPage(login: Action<LogIn, Result<String, ServerFnError>>) -> impl IntoView {
    view! {
        <>
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
            <Login login target=OAuthTarget::GOOGLE/>
            <Login login target=OAuthTarget::REDDIT/>
            <Login login target=OAuthTarget::GITHUB/>
        </div>
        </>
    }
}
