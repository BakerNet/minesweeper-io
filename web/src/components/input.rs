use leptos::*;

#[component]
pub fn TextInput(
    #[prop(optional)] class: &'static str,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] placeholder_owned: String,
    name: &'static str,
) -> impl IntoView {
    let placeholder = if placeholder.len() > 0 {
        placeholder.to_owned()
    } else {
        placeholder_owned
    };
    let class = format!("flex h-10 w-full border border-blue-950 bg-white px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50 flex-1 {}", class);
    view! { <input class=class type="text" placeholder=placeholder name=name/> }
}
