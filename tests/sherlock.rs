use std::{fs::File, io::Read, ops::Range};

use arithmetic_coding::Model;

mod common;

const ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,\n-':()[]#*;\"!?*&é/àâè%@$";

#[derive(Debug, Clone)]
pub struct StringModel;

#[derive(Debug, thiserror::Error)]
#[error("invalid character: {0}")]
pub struct Error(char);

impl Model for StringModel {
    type Symbol = char;
    type ValueError = Error;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Error> {
        if let Some(char) = symbol {
            match ALPHABET.chars().position(|x| &x == char) {
                Some(index) => Ok((index as u32)..(index as u32 + 1)),
                None => Err(Error(*char)),
            }
        } else {
            let alphabet_length = ALPHABET.len() as u32;
            Ok(alphabet_length..(alphabet_length + 1))
        }
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        ALPHABET.chars().nth(value as usize)
    }

    fn max_denominator(&self) -> u32 {
        ALPHABET.len() as u32 + 1
    }
}

#[test]
fn round_trip_u32() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let input = string.chars().collect();

    common::round_trip::<u32, _>(input, StringModel);
}

#[test]
fn round_trip_u64() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let input = string.chars().collect();

    common::round_trip::<u64, _>(input, StringModel);
}

#[test]
fn round_trip_u128() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let input = string.chars().collect();

    common::round_trip::<u128, _>(input, StringModel);
}
