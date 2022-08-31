#[macro_use]
extern crate slog;

pub mod api;
pub mod app;
pub mod cmd;
pub mod model;
pub mod store;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// This is defined as a convenience.
pub type Result<T> = std::result::Result<T, Error>;
