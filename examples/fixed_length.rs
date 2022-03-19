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

#[derive(Clone)]
pub struct MyModel {
    remaining: usize,
}

impl MyModel {
    fn new(length: usize) -> Self {
        Self { remaining: length }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected EOF")]
    UnexpectedEof,
    #[error("Unexpected Symbol")]
    UnexpectedSymbol,
}

impl Model for MyModel {
    type Symbol = Symbol;
    type ValueError = Error;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Result<Range<u32>, Self::ValueError> {
        if self.remaining > 0 {
            match symbol {
                Some(&Symbol::A) => Ok(0..1),
                Some(&Symbol::B) => Ok(1..2),
                Some(&Symbol::C) => Ok(2..3),
                None => Err(Error::UnexpectedEof),
            }
        } else {
            match symbol {
                Some(_) => Err(Error::UnexpectedSymbol),
                None => Ok(0..3),
            }
        }
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        if self.remaining > 0 {
            match value {
                0..1 => Some(Symbol::A),
                1..2 => Some(Symbol::B),
                2..3 => Some(Symbol::C),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    fn max_denominator(&self) -> u32 {
        3
    }

    fn update(&mut self, symbol: Option<&Self::Symbol>) {
        if symbol.is_some() {
            self.remaining -= 1;
        }
    }
}

fn main() {
    let input = [Symbol::A, Symbol::B, Symbol::C];

    let model = MyModel::new(3);

    let output = Vec::default();
    let mut bitwriter = BitWriter::endian(output, BigEndian);
    let mut encoder = Encoder::new(model.clone());

    println!("encoding...");
    encoder.encode_all(input, &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    println!("buffer: {:?}", &buffer);

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);

    println!("\ndecoding...");
    let mut decoder = Decoder::new(model, bitreader).unwrap();
    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        dbg!(symbol);
    }
}
