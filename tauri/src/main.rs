mod app;
mod game;

use app::*;
use leptos::prelude::*;

fn main() {
    #[cfg(debug_assertions)]
    let log_level = log::Level::Debug;
    #[cfg(not(debug_assertions))]
    let log_level = log::Level::Warn;
    _ = console_log::init_with_level(log_level);
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App /> }
    })
}
