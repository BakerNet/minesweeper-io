use leptos::*;
use leptos_router::*;

use crate::components::dark_mode::DarkModeToggle;

use super::FrontendUser;

#[component]
pub fn Header<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    view! {
        <header class="flex items-center justify-between px-4 py-2 border-b dark:border-gray-800">
            <A href="/" class="flex items-center space-x-2">
                <h1 class="text-4xl font-bold text-center text-gray-900 dark:text-gray-200">
                    Minesweeper
                </h1>
            </A>
            <div class="flex items-center space-x-2">
                <Transition fallback=move || {
                    view! {}
                }>
                    <span class="text-lg text-gray-900 dark:text-gray-200">
                        {user
                            .get()
                            .map(|user| {
                                let aclass = "text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500";
                                match user {
                                    None => {
                                        view! {
                                            "Guest ("
                                            <A href="/auth/login" class=aclass>
                                                Log in
                                            </A>
                                            ")"
                                        }
                                            .into_view()
                                    }
                                    Some(user) => {
                                        let name = FrontendUser::display_name_or_anon(
                                            &user.display_name,
                                        );
                                        view! {
                                            {name}
                                            " ("
                                            <A href="/profile" class=aclass>
                                                Profile
                                            </A>
                                            ")"
                                        }
                                            .into_view()
                                    }
                                }
                            })}

                    </span>
                </Transition>
                <DarkModeToggle/>
            </div>
        </header>
    }
}
