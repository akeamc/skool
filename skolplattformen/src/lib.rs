#![doc = include_str!("../README.md")]
#![deny(
    unreachable_pub,
    missing_debug_implementations,
    missing_docs,
    clippy::pedantic
)]
pub mod schedule;
pub mod util;

/// User agent used in this crate.
pub const USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:95.0) Gecko/20100101 Firefox/95.0";
