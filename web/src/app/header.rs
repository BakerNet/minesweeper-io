use leptos::either::*;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_use::{use_color_mode, ColorMode, UseColorModeReturn};

use web_auth::FrontendUser;

use crate::app::background::{BackgroundToggle, BackgroundVariant};
use game_ui::logo;

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

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode();
    view! {
        <button
            id="dark-mode-toggle"
            type="button"
            aria-label="dark mode toggle"
            class="inline-flex items-center justify-center rounded-md text-sm font-medium disabled:pointer-events-none disabled:opacity-50 border border-input bg-slate-100 dark:bg-slate-800 hover:bg-sky-600 hover:text-white dark:hover:bg-sky-700 h-10 px-3 text-gray-900 dark:text-gray-200 cursor-pointer"
            on:click=move |_| {
                match mode() {
                    ColorMode::Dark => set_mode(ColorMode::Light),
                    ColorMode::Light => set_mode(ColorMode::Dark),
                    ColorMode::Auto => {}
                    ColorMode::Custom(_) => {}
                }
            }
        >

            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="h-4 w-4"
            >
                <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"></path>
            </svg>
        </button>
    }
}
