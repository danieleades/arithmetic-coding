use std::ops::Range;

use arithmetic_coding::Model;

mod common;

/// A model for encoding/decoding integers 1..4
#[derive(Clone)]
pub struct MyModel;

/// Invalid symbol error.
///
/// This occurs if an integer is found that is not in the range 1..4
#[derive(Debug, thiserror::Error)]
#[error("invalid symbol: {0}")]
pub struct Error(u8);

impl Model for MyModel {
    type B = u32;
    type Symbol = u8;
    type ValueError = Error;

    /// Given a symbol, return an interval representing the probability of that
    /// symbol occurring.
    ///
    /// Each symbol (plus a 'stop' symbol) is assigned a unique span of the
    /// interval from 0-4. In this case each span has equal probability, so
    /// is the same size.
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Error> {
        match symbol {
            None => Ok(0..1),
            Some(&1) => Ok(1..2),
            Some(&2) => Ok(2..3),
            Some(&3) => Ok(3..4),
            Some(x) => Err(Error(*x)),
        }
    }

    /// For decoding, for each possible value, a symbol is returned.
    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        match value {
            0..1 => None,
            1..2 => Some(1),
            2..3 => Some(2),
            3..4 => Some(3),
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
        4
    }
}

fn main() {
    common::round_trip(MyModel, vec![2, 1, 1, 2, 2, 3, 1]);
}
