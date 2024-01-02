use cfg_if::cfg_if;
use leptos::*;
use regex::Regex;

pub mod app;
pub mod auth;
pub mod error_template;
pub mod fileserv;
pub mod models;
pub mod views;

cfg_if! { if #[cfg(feature = "ssr")]{
    pub mod backend;
}}

pub fn no_prefix_serverfnerror(s: ServerFnError) -> String {
    s.to_string()
        .split(": ")
        .last()
        .expect("ServerFnError String expected to have prefix")
        .to_string()
}

pub fn validate_display_name(name: &str) -> bool {
    let re = Regex::new(r"^[\w]+$").unwrap();
    re.is_match(name) && name.len() >= 3 && name.len() <= 16
}

cfg_if! { if #[cfg(feature = "hydrate")] {
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::app::*;

    #[wasm_bindgen]
    pub fn hydrate() {
        // initializes logging using the `log` crate
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();

        leptos::mount_to_body(App);
    }
}}
