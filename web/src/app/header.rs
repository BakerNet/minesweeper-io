use codee::string::JsonSerdeCodec;
use leptos::either::*;
use leptos::prelude::*;
use leptos_router::components::*;

use crate::{
    cell_class,
    components::{dark_mode::DarkModeToggle, icons::Flag},
    number_class,
};

use super::auth::FrontendUser;

fn logo() -> impl IntoView {
    let white_bg = "bg-white hover:bg-neutral-300";
    let cell_class_1 = cell_class!(number_class!(1), white_bg);
    let cell_class_2 = cell_class!(number_class!(2), white_bg);
    let cell_class_3 = cell_class!(number_class!(3), white_bg);
    let cell_class_4 = cell_class!(number_class!(4), white_bg);
    let cell_class_flag = cell_class!("", "bg-neutral-500 hover:bg-neutral-600/90");
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
                <Flag />
            </span>
            <span class=cell_class_1.clone()>e</span>
            <span class=cell_class_1>r</span>
        </span>
    }
}

#[component]
pub fn Header(user: Resource<Option<FrontendUser>, JsonSerdeCodec>) -> impl IntoView {
    let aclass = "text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500";

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
        <header class="flex flex-wrap space-y-2 space-x-4 items-center justify-between px-4 py-2 border-b border-gray-800">
            <A href="/" attr:class="flex items-center space-x-2">
                <h1>{logo()}</h1>
            </A>
            <A href="/active" attr:class=format!("{} flex items-center space-x-2 text-lg", aclass)>
                "Active Games"
            </A>
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
                <DarkModeToggle />
            </div>
        </header>
    }
}
