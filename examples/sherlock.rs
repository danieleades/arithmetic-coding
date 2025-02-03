use std::{fs::File, io::Read, ops::Range};

use arithmetic_coding::Model;

mod common;

const ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,\n-':()[]#*;\"!?*&é/àâè%@$";

#[derive(Debug, Clone)]
pub struct StringModel {
    alphabet: Vec<char>,
}

impl StringModel {
    #[must_use]
    pub const fn new(alphabet: Vec<char>) -> Self {
        Self { alphabet }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid character: {0}")]
pub struct Error(char);

impl Model for StringModel {
    type B = usize;
    type Symbol = char;
    type ValueError = Error;

    #[allow(clippy::range_plus_one)]
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<usize>, Error> {
        symbol.map_or_else(
            || {
                let alphabet_length = self.alphabet.len();
                Ok(alphabet_length..(alphabet_length + 1))
            },
            |char| {
                self.alphabet
                    .iter()
                    .position(|x| x == char)
                    .map_or(Err(Error(*char)), |index| Ok(index..(index + 1)))
            },
        )
    }

    fn symbol(&self, value: usize) -> Option<Self::Symbol> {
        self.alphabet.get(value).copied()
    }

    fn max_denominator(&self) -> usize {
        self.alphabet.len() + 1
    }
}

fn main() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let input_bytes = input.len();

    let model = StringModel::new(ALPHABET.chars().collect());

    let buffer = common::encode(model.clone(), input.chars());

    let output_bytes = buffer.len();

    println!("input bytes: {input_bytes}");
    println!("output bytes: {output_bytes}");

    #[allow(clippy::cast_precision_loss)]
    let compression_ratio = input_bytes as f32 / output_bytes as f32;
    println!("compression ratio: {compression_ratio}");

    // println!("buffer: {:?}", &buffer);

    let output = common::decode(model, &buffer);

    let mut prefix: String = output.into_iter().take(299).collect();
    prefix.push_str("...");

    println!("{prefix}");
}
