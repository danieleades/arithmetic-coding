//! Arithmetic coding library

#![deny(
    missing_docs,
    clippy::all,
    missing_debug_implementations,
    clippy::cargo
)]
#![warn(clippy::pedantic)]
#![feature(int_log)]
#![feature(associated_type_defaults)]

mod model;
pub use model::Model;

mod encoder;
pub use encoder::Encoder;

mod decoder;
pub use decoder::Decoder;

mod bitstore;
pub use bitstore::BitStore;

pub mod fixed_length;

/// Errors that can occur during encoding/decoding
#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
    /// Io error when reading/writing bits from a stream
    Io(#[from] std::io::Error),

    /// Invalid symbol
    ValueError(E),
}
