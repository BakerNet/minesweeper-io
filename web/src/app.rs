mod auth;
mod error_template;
mod footer;
mod header;
mod home;
mod login;
mod minesweeper;
mod profile;
mod root;

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use root::App;

#[cfg(feature = "ssr")]
pub use auth::{FrontendUser, OAuthTarget};
