use leptos::*;

#[component]
pub fn Button(
    #[prop(optional)] class: &'static str,
    #[prop(optional)] btn_type: &'static str,
    children: Children,
) -> impl IntoView {
    let class = format!("inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2 {}", class);
    view! {
        <button class=class type=btn_type>
            {children()}
        </button>
    }
}
