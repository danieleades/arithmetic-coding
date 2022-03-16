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
