use leptos::*;

#[component]
pub fn Button(
    #[prop(optional)] class: &'static str,
    #[prop(optional)] btn_type: &'static str,
    children: Children,
) -> impl IntoView {
    let class = format!("inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 bg-neutral-500 text-neutral-50 hover:bg-neutral-600/90 h-10 px-4 py-2 {}", class);
    view! {
        <button class=class type=btn_type>
            {children()}
        </button>
    }
}
