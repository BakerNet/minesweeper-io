use codee::string::JsonSerdeWasmCodec;
use game_ui::icons::Mine;
use leptos::prelude::*;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum BackgroundVariant {
    #[default]
    None,
    FloatingMines,
    GridPattern,
    Gradient,
}

impl BackgroundVariant {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::FloatingMines,
            Self::FloatingMines => Self::GridPattern,
            Self::GridPattern => Self::Gradient,
            Self::Gradient => Self::None,
        }
    }
}

#[component]
pub fn AnimatedBackground(variant: Signal<BackgroundVariant>) -> impl IntoView {
    view! {
        {move || match variant.get() {
            BackgroundVariant::None => view! { <div></div> }.into_any(),
            BackgroundVariant::FloatingMines => view! { <FloatingMinesBackground /> }.into_any(),
            BackgroundVariant::GridPattern => view! { <GridPatternBackground /> }.into_any(),
            BackgroundVariant::Gradient => view! { <GradientBackground /> }.into_any(),
        }}
    }
}

#[component]
fn FloatingMinesBackground() -> impl IntoView {
    view! {
        <div class="fixed inset-0 pointer-events-none overflow-hidden">
            // Large mines - slow movement
            <div class="absolute animate-float-slow opacity-40 w-16 h-16 floating-mine" style="top: 20%; left: 15%; animation-delay: 0s;">
                <Mine />
            </div>
            <div class="absolute animate-float-slow opacity-35 w-15 h-15 floating-mine" style="top: 75%; left: 85%; animation-delay: 4s;">
                <Mine />
            </div>
            // Medium mines - medium movement
            <div class="absolute animate-float-medium opacity-35 w-14 h-14 floating-mine" style="top: 60%; left: 80%; animation-delay: 2s;">
                <Mine />
            </div>
            <div class="absolute animate-float-medium opacity-30 w-13 h-13 floating-mine" style="top: 10%; left: 70%; animation-delay: 1.5s;">
                <Mine />
            </div>
            <div class="absolute animate-float-medium opacity-32 w-12 h-12 floating-mine" style="top: 45%; left: 5%; animation-delay: 3.5s;">
                <Mine />
            </div>
            <div class="absolute animate-float-medium opacity-28 w-14 h-14 floating-mine" style="top: 85%; left: 60%; animation-delay: 5s;">
                <Mine />
            </div>
            // Small mines - fast movement
            <div class="absolute animate-float-fast opacity-30 w-12 h-12 floating-mine" style="top: 40%; left: 60%; animation-delay: 1s;">
                <Mine />
            </div>
            <div class="absolute animate-float-fast opacity-25 w-8 h-8 floating-mine" style="top: 35%; left: 10%; animation-delay: 2.5s;">
                <Mine />
            </div>
            <div class="absolute animate-float-fast opacity-27 w-9 h-9 floating-mine" style="top: 15%; left: 45%; animation-delay: 0.5s;">
                <Mine />
            </div>
            <div class="absolute animate-float-fast opacity-22 w-10 h-10 floating-mine" style="top: 70%; left: 25%; animation-delay: 4.5s;">
                <Mine />
            </div>
            <div class="absolute animate-float-fast opacity-25 w-8 h-8 floating-mine" style="top: 55%; left: 90%; animation-delay: 6s;">
                <Mine />
            </div>
            // Additional depth mines - mixed speeds
            <div class="absolute animate-float-slow opacity-25 w-10 h-10 floating-mine" style="top: 80%; left: 30%; animation-delay: 3s;">
                <Mine />
            </div>
            <div class="absolute animate-float-medium opacity-26 w-11 h-11 floating-mine" style="top: 25%; left: 88%; animation-delay: 2.8s;">
                <Mine />
            </div>
            <div class="absolute animate-float-slow opacity-20 w-9 h-9 floating-mine" style="top: 5%; left: 20%; animation-delay: 1.2s;">
                <Mine />
            </div>
            <div class="absolute animate-float-fast opacity-24 w-7 h-7 floating-mine" style="top: 90%; left: 8%; animation-delay: 3.8s;">
                <Mine />
            </div>
            <div class="absolute animate-float-medium opacity-29 w-13 h-13 floating-mine" style="top: 30%; left: 75%; animation-delay: 5.5s;">
                <Mine />
            </div>
        </div>
    }
}

#[component]
fn GridPatternBackground() -> impl IntoView {
    view! {
        <div class="fixed inset-0 pointer-events-none">
            <div class="absolute inset-0 opacity-30 bg-grid-pattern dark:opacity-40"></div>
        </div>
    }
}

#[component]
fn GradientBackground() -> impl IntoView {
    view! {
        <div class="fixed inset-0 pointer-events-none">
            <div class="absolute inset-0 gradient-layer-1"></div>
            <div class="absolute inset-0 gradient-layer-2"></div>
            <div class="absolute inset-0 gradient-layer-3"></div>
        </div>
    }
}

#[component]
pub fn BackgroundToggle() -> impl IntoView {
    let storage_options =
        UseStorageOptions::<BackgroundVariant, serde_json::Error, JsValue>::default()
            .initial_value(BackgroundVariant::FloatingMines)
            .delay_during_hydration(true);
    let (background_variant, set_background_variant, _) =
        use_local_storage_with_options::<BackgroundVariant, JsonSerdeWasmCodec>(
            "background_variant",
            storage_options,
        );

    view! {
        <button
            id="background-toggle"
            type="button"
            aria-label="background toggle"
            class="inline-flex items-center justify-center rounded-md text-sm font-medium disabled:pointer-events-none disabled:opacity-50 border border-input bg-transparent hover:bg-gray-700 hover:text-gray-50 h-10 px-3 text-gray-900 dark:text-gray-200"
            on:click=move |_| {
                set_background_variant(background_variant.get_untracked().next());
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
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <circle cx="9" cy="9" r="2"></circle>
                <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"></path>
            </svg>
        </button>
    }
}
