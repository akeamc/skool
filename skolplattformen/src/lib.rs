#![doc = include_str!("../README.md")]
#![warn(
    unreachable_pub,
    missing_debug_implementations,
    missing_docs,
    clippy::pedantic
)]

mod client;
pub mod schedule;
mod session;
mod util;

pub use client::*;
pub use session::*;

/// An error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Bad login credentials, i.e. username and password.
    #[error("bad credentials")]
    BadCredentials,

    /// Some HTTP request failed.
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    /// The scraping failed, most likely due to some unexpected HTML.
    #[error("scraping failed: {details}")]
    ScrapingFailed {
        /// Detailed error information (human readable).
        details: String,
    },
}

/// Skolplattformen result.
pub type Result<T, E = Error> = core::result::Result<T, E>;
