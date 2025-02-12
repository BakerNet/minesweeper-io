use leptos::prelude::*;
use regex::Regex;

use web_auth::FrontendUser;

use game_ui::{button_class, input_class};

#[cfg(feature = "ssr")]
use super::get_user;
#[cfg(feature = "ssr")]
use axum_login::AuthUser;
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

fn no_prefix_serverfnerror(s: ServerFnError) -> String {
    s.to_string()
        .split(": ")
        .last()
        .expect("ServerFnError String expected to have prefix")
        .to_string()
}

fn validate_display_name(name: &str) -> bool {
    let re = Regex::new(r"^[\w]+$").unwrap();
    re.is_match(name) && name.len() >= 3 && name.len() <= 16
}

#[server]
pub async fn set_display_name(display_name: String) -> Result<String, ServerFnError> {
    if !validate_display_name(&display_name) {
        return Err(ServerFnError::new("Display name not valid".to_string()));
    }
    let user = get_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Unable to find user".to_string()))?;
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
        .map_err(|_| ServerFnError::new("Unable to update display name".to_string()))
}

#[component]
pub fn SetDisplayName(user: FrontendUser, user_updated: WriteSignal<String>) -> impl IntoView {
    let set_display_name = ServerAction::<SetDisplayName>::new();
    let (name_err, set_name_err) = signal::<Option<String>>(None);

    let on_submit = move |ev| {
        let data = SetDisplayName::from_event(&ev);
        if data.is_err() || !validate_display_name(&data.unwrap().display_name) {
            ev.prevent_default();
            set_name_err(Some("Display name not valid".to_string()));
        }
    };

    Effect::new(move |_| match set_display_name.value().get() {
        Some(Ok(name)) => {
            user_updated(name);
            set_name_err(None);
        }
        Some(Err(e)) => set_name_err(Some(
            no_prefix_serverfnerror(e) + ". This display name may already be taken",
        )),
        _ => {}
    });

    let curr_name = FrontendUser::display_name_or_anon(user.display_name.as_ref(), true);

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
                attr:class="flex space-x-2"
            >
                <input
                    class=input_class!()
                    type="text"
                    id="set_display_name_display_name"
                    name="display_name"
                    placeholder=curr_name
                />
                <button type="submit" class=button_class!() disabled=set_display_name.pending()>
                    "Set display name"
                </button>
            </ActionForm>
        </div>
    }
}
