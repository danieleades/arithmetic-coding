use std::{convert::Infallible, ops::Range};

use arithmetic_coding::Model;

mod common;

#[derive(Debug)]
pub enum Symbol {
    A,
    B,
    C,
}

#[derive(Clone)]
pub struct MyModel;

impl Model for MyModel {
    type B = u32;
    type Symbol = Symbol;
    type ValueError = Infallible;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Infallible> {
        Ok(match symbol {
            None => 0..1,
            Some(&Symbol::A) => 1..2,
            Some(&Symbol::B) => 2..3,
            Some(&Symbol::C) => 3..4,
        })
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        match value {
            0..1 => None,
            1..2 => Some(Symbol::A),
            2..3 => Some(Symbol::B),
            3..4 => Some(Symbol::C),
            _ => unreachable!(),
        }
    }

    fn max_denominator(&self) -> u32 {
        4
    }
}

fn main() {
    common::round_trip(MyModel, vec![Symbol::A, Symbol::B, Symbol::C]);
}
