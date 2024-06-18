use cfg_if::cfg_if;

mod app;
mod components;
mod messages;
mod models;

#[cfg(feature = "ssr")]
pub mod backend;

cfg_if! { if #[cfg(feature = "hydrate")] {
    use leptos::*;
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::App;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        #[cfg(debug_assertions)]
        let log_level = log::Level::Debug;
        #[cfg(not(debug_assertions))]
        let log_level = log::Level::Warn;
        _ = console_log::init_with_level(log_level);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(App);
    }
}}
