#![allow(clippy::module_name_repetitions)]

pub mod dynamic;
pub mod fixed;
mod parse_static;
mod serde_helper;

pub use parse_static::parse_static;
