use std::{convert::Infallible, ops::Range};

use arithmetic_coding::fixed_length;

mod common;

/// The possible symbols
#[derive(Debug)]
pub enum Symbol {
    A,
    B,
    C,
}

/// A model for encoding/decoding a set of 3 symbols from {A, B, C}
#[derive(Clone)]
pub struct MyModel;

impl fixed_length::Model for MyModel {
    type B = u32;
    type Symbol = Symbol;
    type ValueError = Infallible;

    /// Given a symbol, return a unique interval representing the probability of
    /// that symbol occurring.
    ///
    /// Since the number of symbols in the message is fixed, no 'stop' symbol is
    /// needed.
    fn probability(&self, symbol: &Self::Symbol) -> Result<Range<u32>, Self::ValueError> {
        match symbol {
            Symbol::A => Ok(0..1),
            Symbol::B => Ok(1..2),
            Symbol::C => Ok(2..3),
        }
    }

    /// For decoding, for each possible value, a symbol is returned.
    fn symbol(&self, value: u32) -> Self::Symbol {
        match value {
            0..1 => Symbol::A,
            1..2 => Symbol::B,
            2..3 => Symbol::C,
            _ => unreachable!(),
        }
    }

    /// The maximum denominator used for probability ranges.
    ///
    /// The trait also includes a 'denominator' method, which is allowed to vary
    /// for each symbol, but must never exceed `max_denominator`.
    /// For non-adaptive models, this value is the same as `max_denominator`
    /// (and this is the default value of the trait method).
    fn max_denominator(&self) -> u32 {
        3
    }

    /// The total number of symbols to encode.
    ///
    /// Because the number of symbols is known ahead of time, we can take
    /// advantage of this to optimise the compression by removing the need
    /// for a 'stop' symbol.
    fn length(&self) -> usize {
        3
    }
}

fn main() {
    let input = vec![Symbol::A, Symbol::B, Symbol::C];
    let model = fixed_length::Wrapper::new(MyModel);

    common::round_trip(model, input);
}
