#[cfg(feature = "ssr")]
mod cache;
#[cfg(feature = "ssr")]
mod game_manager;
mod messages;
#[cfg(feature = "ssr")]
pub mod models;
#[cfg(feature = "ssr")]
mod websocket;

#[cfg(feature = "ssr")]
pub use game_manager::GameManager;
pub use messages::{ClientMessage, GameMessage};
#[cfg(feature = "ssr")]
pub use websocket::{websocket_handler, ExtractGameManager};
