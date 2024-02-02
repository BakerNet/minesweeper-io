use leptos::*;
use leptos_use::{use_color_mode, ColorMode, UseColorModeReturn};

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode();
    view! {
        <button
            type="button"
            class="inline-flex items-center justify-center rounded-md text-sm font-medium disabled:pointer-events-none disabled:opacity-50 border border-input bg-transparent hover:bg-gray-700 hover:text-gray-50 h-10 px-3 text-gray-900 dark:text-gray-200"
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
