//! Rust wrapper for interacting with the Extel API
//!
//! See the [API documentation](https://webservices.api.exetel.com.au/doc/) for more information.

mod auth;
mod client;
pub mod customer;

pub use auth::Authorization;
pub use client::Client;
use client::Query;
