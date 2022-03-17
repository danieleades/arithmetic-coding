#![feature(exclusive_range_pattern)]
#![feature(never_type)]

use std::ops::Range;

use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

#[derive(Debug)]
pub enum Symbol {
    A,
    B,
    C,
}

pub struct MyModel;

impl Model for MyModel {
    type Symbol = Symbol;
    type ValueError = !;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, !> {
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
    let input = [Symbol::A, Symbol::B, Symbol::C];

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
