use leptos::prelude::*;

use crate::components::socials::{GitHubSocial, LinkedInSocial};

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="relative h-16 space-y-2 px-4 py-2 border-t border-gray-800">
            <div class="flex items-center justify-center mx-auto w-8/12 h-full text-gray-900 dark:text-gray-100">
                <span>
                    "Developed by a "
                    <a
                        class="text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500"
                        href="https://hansbaker.com"
                    >
                        "minesweeper nerd"
                    </a>
                </span>
            </div>
            <div class="absolute top-2 right-2 flex items-center space-x-2">
                <LinkedInSocial />
                <GitHubSocial />
            </div>
        </footer>
    }
}
