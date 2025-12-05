//! Core traits for the [`arithmetic-coding`](https://github.com/danieleades/arithmetic-coding) crate

#![deny(missing_docs, missing_debug_implementations)]

mod bitstore;
pub use bitstore::BitStore;

mod model;
pub use model::{Model, fixed_length, max_length, one_shot};
