//! Slack integration for CS-CLI
//!
//! Provides Slack workspace access using the common abstractions for
//! authentication, HTTP clients, and data models.

pub mod api;
pub mod auth;
pub mod explorer;
pub mod models;
pub mod test;

pub use api::*;
pub use auth::*;
pub use explorer::*;
pub use models::*;
pub use test::*;
