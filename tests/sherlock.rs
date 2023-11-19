use std::{fs::File, io::Read, ops::Range};

use arithmetic_coding::Model;

mod common;

const ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,\n\r-':()[]#*;\"!?*&é/àâè%@$";

#[derive(Debug, Clone)]
pub struct StringModel;

#[derive(Debug, thiserror::Error)]
#[error("invalid character: {0}")]
pub struct Error(char);

impl Model for StringModel {
    type B = usize;
    type Symbol = char;
    type ValueError = Error;

    #[allow(clippy::range_plus_one)]
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<Self::B>, Error> {
        symbol.map_or_else(
            || Ok(ALPHABET.len()..(ALPHABET.len() + 1)),
            |char| {
                ALPHABET
                    .chars()
                    .position(|x| &x == char)
                    .ok_or(Error(*char))
                    .map(|index| index..(index + 1))
            },
        )
    }

    fn symbol(&self, value: Self::B) -> Option<Self::Symbol> {
        ALPHABET.chars().nth(value)
    }

    fn max_denominator(&self) -> Self::B {
        ALPHABET.len() + 1
    }
}

#[test]
fn round_trip() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let input: Vec<_> = string.chars().collect();

    common::round_trip(StringModel, &input);
}
