//! Arithmetic coding library

#![deny(
    missing_docs,
    clippy::all,
    missing_debug_implementations,
    clippy::cargo
)]
#![warn(clippy::pedantic)]

pub use arithmetic_coding_core::{fixed_length, BitStore, Model};

mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

/// Errors that can occur during encoding/decoding
#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
    /// Io error when reading/writing bits from a stream
    Io(#[from] std::io::Error),

    /// Invalid symbol
    ValueError(E),
}
