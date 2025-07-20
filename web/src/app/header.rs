use leptos::either::*;
use leptos::prelude::*;
use leptos_router::components::*;

use web_auth::FrontendUser;

use game_ui::{logo, DarkModeToggle, BackgroundToggle, BackgroundVariant};

#[component]
pub fn Header(
    user: Resource<Option<FrontendUser>>,
    set_background_variant: WriteSignal<BackgroundVariant>,
) -> impl IntoView {
    let aclass = "text-sky-700 dark:text-sky-500 hover:text-sky-900 dark:hover:text-sky-400 font-medium";

    let user_info = move |user: Option<FrontendUser>| match user {
        None => Either::Left(view! {
            <span>
                "Guest (" <A href="/auth/login" attr:class=aclass>
                    "Log in"
                </A> ")"
            </span>
        }),
        Some(user) => {
            let name = FrontendUser::display_name_or_anon(user.display_name.as_ref(), true);
            Either::Right(view! {
                <span>
                    {name} " (" <A href="/profile" attr:class=aclass>
                        "Profile"
                    </A> ")"
                </span>
            })
        }
    };
    view! {
        <header class="flex flex-wrap space-y-2 space-x-4 items-center justify-between px-4 py-2 border-b border-gray-800 bg-neutral-200/50 dark:bg-gray-950/50">
            <A href="/" attr:class="flex items-center space-x-2">
                <h1>{logo()}</h1>
            </A>
            <div class="flex items-center space-x-2">
                <A href="/active" attr:class=format!("{} text-lg", aclass)>
                    "Active Games"
                </A>
                <span class="text-gray-950 dark:text-gray-100">"|"</span>
                <A href="/recent" attr:class=format!("{} text-lg", aclass)>
                    "Recent Games"
                </A>
            </div>
            <div class="flex grow justify-end items-center space-x-2">
                <Transition fallback=move || ()>
                    {move || Suspend::new(async move {
                        let user = user.await;
                        let user = user_info(user);
                        view! {
                            <span class="text-lg text-gray-900 dark:text-gray-200">{user}</span>
                        }
                    })}

                </Transition>
                <BackgroundToggle set_background_variant />
                <DarkModeToggle />
            </div>
        </header>
    }
}

