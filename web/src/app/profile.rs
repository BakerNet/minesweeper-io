use cfg_if::cfg_if;

use leptos::*;
use leptos_router::*;

use super::{auth::LogOut, FrontendUser};
use crate::{
    components::{button::Button, input::TextInput},
    no_prefix_serverfnerror, validate_display_name,
};

cfg_if! { if #[cfg(feature="ssr")] {
    use axum_login::AuthUser;
    use super::auth::get_user;
    use crate::backend::users::AuthSession;
}}

#[component]
pub fn Profile(
    logout: Action<LogOut, Result<(), ServerFnError>>,
    user: FrontendUser,
    user_updated: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
            <SetDisplayName user user_updated/>
            <div class="w-full max-w-xs h-6">
                <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                    <hr class="w-full"/>
                </span>
            </div>
            <LogOut logout/>
        </div>
    }
}

#[server(SetDisplayName, "/api")]
async fn set_display_name(display_name: String) -> Result<String, ServerFnError> {
    if !validate_display_name(&display_name) {
        return Err(ServerFnError::ServerError(
            "Display name not valid".to_string(),
        ));
    }
    let user = get_user()
        .await?
        .ok_or_else(|| ServerFnError::ServerError("Unable to find user".to_string()))?;
    if let Some(name) = &user.display_name {
        if name == &display_name {
            return Ok(display_name);
        }
    }
    let auth_session = use_context::<AuthSession>().unwrap();
    auth_session
        .backend
        .update_user_display_name(user.id(), &display_name)
        .await
        .map(|_| display_name)
        .map_err(|_| ServerFnError::ServerError("Unable to update display name".to_string()))
}

#[component]
fn SetDisplayName(user: FrontendUser, user_updated: WriteSignal<String>) -> impl IntoView {
    let set_display_name = create_server_action::<SetDisplayName>();
    let (name_err, set_name_err) = create_signal::<Option<String>>(None);

    let on_submit = move |ev| {
        let data = SetDisplayName::from_event(&ev);
        if data.is_err() || !validate_display_name(&data.unwrap().display_name) {
            ev.prevent_default();
            set_name_err(Some("Display name not valid".to_string()));
        }
    };

    create_effect(move |_| match set_display_name.value().get() {
        Some(Ok(name)) => {
            user_updated(name);
            set_name_err(None);
        }
        Some(Err(e)) => set_name_err(Some(
            no_prefix_serverfnerror(e) + ". This display name may already be taken",
        )),
        _ => {}
    });

    let curr_name = FrontendUser::display_name_or_anon(&user.display_name);

    view! {
        <div class="flex flex-col space-y-2 w-full max-w-xs">
            <span class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                {curr_name.clone()}
            </span>
            {move || {
                name_err
                    .get()
                    .map(|s| {
                        view! {
                            <span class="text-sm font-medium leading-none text-red-500">{s}</span>
                        }
                    })
            }}

            <ActionForm
                action=set_display_name
                on:submit=move |e| on_submit(e.into())
                class="flex space-x-2"
            >
                <TextInput name="display_name" placeholder_owned=curr_name/>
                <Button btn_type="submit">"Set display name"</Button>
            </ActionForm>
        </div>
    }
}
