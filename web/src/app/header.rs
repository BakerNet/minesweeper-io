use leptos::*;
use leptos_router::*;

use super::FrontendUser;

#[component]
pub fn Header<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    view! {
        <header>
            <A href="/">
                <h2>Minesweeper</h2>
            </A>
            <Transition fallback=move || {
                view! {}
            }>
                {user
                    .get()
                    .map(|user| match user {
                        None => {
                            view! { <span>"Guest (" <A href="/auth/login">Log in</A> ")"</span> }
                                .into_view()
                        }
                        Some(user) => {
                            let name = FrontendUser::display_name_or_anon(&user.display_name);
                            view! { <span>{name} " (" <A href="/profile">Profile</A> ")"</span> }
                                .into_view()
                        }
                    })}

            </Transition>

        </header>
    }
}
