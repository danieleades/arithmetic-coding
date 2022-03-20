#![feature(exclusive_range_pattern)]
#![feature(never_type)]

use std::ops::Range;

use arithmetic_coding::{fixed_length, Decoder, Encoder};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

#[derive(Debug)]
pub enum Symbol {
    A,
    B,
    C,
}

#[derive(Clone)]
pub struct MyModel;

impl fixed_length::Model for MyModel {
    type Symbol = Symbol;
    type ValueError = !;

    fn probability(&self, symbol: &Self::Symbol) -> Result<Range<u32>, Self::ValueError> {
        match symbol {
            Symbol::A => Ok(0..1),
            Symbol::B => Ok(1..2),
            Symbol::C => Ok(2..3),
        }
    }

    fn symbol(&self, value: u32) -> Self::Symbol {
        match value {
            0..1 => Symbol::A,
            1..2 => Symbol::B,
            2..3 => Symbol::C,
            _ => unreachable!(),
        }
    }

    fn max_denominator(&self) -> u32 {
        3
    }

    fn length(&self) -> usize {
        3
    }
}

fn main() {
    let input = [Symbol::A, Symbol::B, Symbol::C];

    let model = fixed_length::Wrapper::new(MyModel);

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
