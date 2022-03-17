use std::{fs::File, io::Read, ops::Range};

use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

const ALPHABET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,\n-':()[]#*;\"!?*&é/àâè%@$";

#[derive(Debug)]
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
fn round_trip() {
    let mut file = File::open("./resources/sherlock.txt").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();

    let mut bitwriter = BitWriter::endian(Vec::new(), BigEndian);

    let mut encoder = Encoder::new(StringModel);

    encoder.encode(input.chars(), &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);
    let mut decoder = Decoder::new(StringModel, bitreader).unwrap();
    let mut output = String::new();

    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        output.push(symbol);
    }

    assert_eq!(input, output);
}
