use std::{convert::Infallible, ops::Range};

use arithmetic_coding::max_length;
use test_case::test_case;

mod common;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Symbol {
    A,
    B,
    C,
}

/// This model encodes a variable number of symbols, up to a maximum of 3.
///
/// By taking advantage of the fact that the maximum number of symbols is known
/// ahead of time, compression is improved compared to a fully variable-length
/// model.
#[derive(Clone)]
pub struct MyModel;

impl max_length::Model for MyModel {
    type B = u32;
    type Symbol = Symbol;
    type ValueError = Infallible;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Self::ValueError> {
        match symbol {
            Some(Symbol::A) => Ok(0..1),
            Some(Symbol::B) => Ok(1..2),
            Some(Symbol::C) => Ok(2..3),
            None => Ok(3..4),
        }
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        match value {
            0..1 => Some(Symbol::A),
            1..2 => Some(Symbol::B),
            2..3 => Some(Symbol::C),
            3..4 => None,
            _ => unreachable!(),
        }
    }

    fn max_denominator(&self) -> u32 {
        4
    }

    fn max_length(&self) -> usize {
        3
    }
}

#[test_case(&[Symbol::A, Symbol::B] ; "shorter")]
#[test_case(&[Symbol::A, Symbol::B, Symbol::C] ; "exact")]
#[test_case(&[Symbol::A, Symbol::B, Symbol::C, Symbol::C] => panics "UnexpectedSymbol" ; "longer")]
fn round_trip(input: &[Symbol]) {
    common::round_trip(max_length::Wrapper::new(MyModel), input);
}
