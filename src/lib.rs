//! Arithmetic coding library

#![deny(missing_docs, clippy::all, missing_debug_implementations)]
#![warn(clippy::pedantic)]
#![feature(int_log)]

mod model;
pub use model::Model;

mod encoder;
pub use encoder::Encoder;

mod decoder;
pub use decoder::Decoder;

mod bitstore;
pub use bitstore::BitStore;

mod util;

/// Errors that can occur during encoding/decoding
#[derive(Debug, thiserror::Error)]
pub enum Error<E> {
    /// Io error when reading/writing bits from a stream
    Io(#[from] std::io::Error),

    /// Invalid symbol
    ValueError(E),
}
