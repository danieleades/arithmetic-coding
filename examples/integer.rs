#![feature(exclusive_range_pattern)]

use std::ops::Range;

use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

pub struct MyModel;

#[derive(Debug, thiserror::Error)]
#[error("invalid symbol: {0}")]
pub struct Error(u8);

impl Model for MyModel {
    type Symbol = u8;
    type ValueError = Error;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Error> {
        match symbol {
            None => Ok(0..1),
            Some(&1) => Ok(1..2),
            Some(&2) => Ok(2..3),
            Some(&3) => Ok(2..4),
            Some(x) => Err(Error(*x)),
        }
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        match value {
            0..1 => None,
            1..2 => Some(1),
            2..3 => Some(2),
            3..4 => Some(3),
            _ => unreachable!(),
        }
    }

    fn denominator(&self) -> u32 {
        4
    }
}

fn main() {
    let input = [2, 1, 1, 2, 2];

    let output = Vec::default();
    let mut bitwriter = BitWriter::endian(output, BigEndian);
    let mut encoder = Encoder::new(MyModel);

    println!("encoding...");
    encoder.encode(input, &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    println!("buffer: {:?}", &buffer);

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);

    println!("\ndecoding...");
    let mut decoder = Decoder::new(MyModel, bitreader).unwrap();
    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        dbg!(symbol);
    }
}
