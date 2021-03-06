//! Core traits for the [`arithmetic-coding`](https://github.com/danieleades/arithmetic-coding) crate

#![deny(
    missing_docs,
    clippy::all,
    missing_debug_implementations,
    clippy::cargo
)]
#![warn(clippy::pedantic)]
#![feature(int_log)]
#![feature(associated_type_defaults)]

mod bitstore;
pub use bitstore::BitStore;

mod model;
pub use model::Model;

pub mod fixed_length;
