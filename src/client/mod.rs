//! WebSocket chat client implementation.

mod domain;
mod formatter;
mod runner;
mod session;
mod ui;

pub use runner::run_client;
