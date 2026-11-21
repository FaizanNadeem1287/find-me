//! Find-Me Desktop User Interface (Pure Rust / Iced)
//!
//! Provides the Elm-architecture model, views, navigation, themes,
//! and virtualized table components for the Find-Me Linux application.

pub mod app;
pub mod message;
pub mod state;
pub mod theme;

pub use app::FindMeApp;
pub use message::{ColumnId, Message, ScreenId};
pub use state::AppState;
pub use theme::AppTheme;
