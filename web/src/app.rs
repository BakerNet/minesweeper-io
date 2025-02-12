mod auth;
mod error_template;
mod footer;
mod header;
mod home;
mod login;
mod minesweeper;
mod profile;
mod root;

#[cfg(feature = "ssr")]
pub use root::shell;
pub use root::App;
