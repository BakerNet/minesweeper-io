use leptos::prelude::*;

#[component]
pub fn ActiveGames() -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col items-center justify-center w-full max-w-4xl py-12 px-4 space-y-4 mx-auto">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Join Multiplayer Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full">
                <div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div>
            </div>
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Watch Active Games"</h1>
            <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 w-full">
                <div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div><div class="h-16 w-full">Test</div>
            </div>
        </div>
    }
}
