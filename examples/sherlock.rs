use std::{fs::File, io::Read, ops::Range};

use arithmetic_coding::Model;

mod common;

const ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,\n-':()[]#*;\"!?*&é/àâè%@$";

const ALPHABET_LEN: usize = ALPHABET.len();

#[derive(Debug, Clone)]
pub struct StringModel<'a, const ALPHABET_LEN: usize> {
    alphabet: &'a str,
}

impl<'a, const ALPHABET_LEN: usize> StringModel<'a, ALPHABET_LEN> {
    #[must_use]
    pub fn new(alphabet: &'a str) -> Self {
        Self { alphabet }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid character: {0}")]
pub struct Error(char);

impl<'a, const ALPHABET_LEN: usize> Model for StringModel<'a, ALPHABET_LEN> {
    type B = usize;
    type Symbol = char;
    type ValueError = Error;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<usize>, Error> {
        if let Some(char) = symbol {
            match self.alphabet.chars().position(|x| x == *char) {
                Some(index) => Ok(index..(index + 1)),
                None => Err(Error(*char)),
            }
        } else {
            let alphabet_length = self.alphabet.len();
            Ok(alphabet_length..(alphabet_length + 1))
        }
    }

    fn symbol(&self, value: usize) -> Option<Self::Symbol> {
        self.alphabet.chars().nth(value)
    }

    const MAX_DENOMINATOR: Self::B = ALPHABET_LEN + 1;
}

fn main() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let input_bytes = input.bytes().len();

    let model = StringModel::<ALPHABET_LEN>::new(ALPHABET);

    let buffer = common::encode(model.clone(), input.chars());

    let output_bytes = buffer.len();

    println!("input bytes: {input_bytes}");
    println!("output bytes: {output_bytes}");

    println!(
        "compression ratio: {}",
        input_bytes as f32 / output_bytes as f32
    );

    // println!("buffer: {:?}", &buffer);

    let output = common::decode(model, &buffer);

    let mut prefix: String = output.into_iter().take(299).collect();
    prefix.push_str("...");

    println!("{prefix}");
}
