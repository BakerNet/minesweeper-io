use leptos::*;
use leptos_router::*;

use crate::{
    app::minesweeper::cell::{cell_class, number_class},
    components::{dark_mode::DarkModeToggle, icons::Flag},
};

use super::FrontendUser;

fn logo() -> impl IntoView {
    let white_bg = "bg-white hover:bg-neutral-300";
    let cell_class_1 = cell_class(&number_class(1), white_bg);
    let cell_class_2 = cell_class(&number_class(2), white_bg);
    let cell_class_3 = cell_class(&number_class(3), white_bg);
    let cell_class_4 = cell_class(&number_class(4), white_bg);
    let cell_class_flag = cell_class("", "bg-neutral-500 hover:bg-neutral-600/90");
    view! {
        <span class="whitespace-nowrap">
            <span class=cell_class_4.clone()>M</span>
            <span class=cell_class_2.clone()>i</span>
            <span class=cell_class_3.clone()>n</span>
            <span class=cell_class_3>e</span>
            <span class=cell_class_4>s</span>
            <span class=cell_class_2.clone()>w</span>
            <span class=cell_class_2>e</span>
            <span class=cell_class_1.clone()>e</span>
            <span class=cell_class_flag>
                <Flag/>
            </span>
            <span class=cell_class_1.clone()>e</span>
            <span class=cell_class_1>r</span>
        </span>
    }
}

#[component]
pub fn Header<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    let user_info = move |user: Option<FrontendUser>| {
        let aclass = "text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500";
        match user {
            None => view! {
                "Guest ("
                <A href="/auth/login" class=aclass>
                    Log in
                </A>
                ")"
            }
            .into_view(),
            Some(user) => {
                let name = FrontendUser::display_name_or_anon(&user.display_name, true);
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
    };
    view! {
        <header class="flex flex-wrap space-y-2 items-center justify-between px-4 py-2 border-b border-gray-800">
            <A href="/" class="flex items-center space-x-2">
                <h1>{logo()}</h1>
            </A>
            <div class="flex items-center space-x-2">
                <Transition fallback=move || {
                    view! {}
                }>
                    <span class="text-lg text-gray-900 dark:text-gray-200">
                        {user
                            .get()
                            .map(user_info)}

                    </span>
                </Transition>
                <DarkModeToggle/>
            </div>
        </header>
    }
}
