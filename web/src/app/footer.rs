use leptos::*;

use crate::components::socials::{GitHubSocial, LinkedInSocial};

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="relative h-16 space-y-2 px-4 py-2 border-t border-gray-800">
            <div class="flex items-center justify-center mx-auto w-8/12 h-full space-x-2 text-gray-900 dark:text-gray-100">
                Developed by a minesweeper nerd
            </div>
            <div class="absolute top-2 right-2 flex items-center space-x-2">
                <LinkedInSocial />
                <GitHubSocial />
            </div>
        </footer>
    }
}
