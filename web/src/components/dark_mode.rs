use leptos::*;

#[component]
pub fn DarkModeToggle() -> impl IntoView {
    view! {
        <button
            type="button"
            aria-pressed="false"
            data-state="off"
            class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=on]:bg-accent data-[state=on]:text-accent-foreground border border-input bg-transparent hover:bg-accent hover:text-accent-foreground h-10 px-3 text-gray-900 dark:text-gray-200"
            aria-label="Toggle dark mode"
            // todo
            on:click=move |_| {}
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
