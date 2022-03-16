#![feature(exclusive_range_pattern)]

use std::ops::Range;

use arithmetic_coding::Decoder;
use arithmetic_coding::{Encoder, Model};
use bitstream_io::BigEndian;
use bitstream_io::BitWriter;
use bitstream_io::{BitReader, BitWrite};

pub struct MyModel;

impl Model for MyModel {
    type Symbol = u8;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Range<u32> {
        match symbol {
            None => 0..1,
            Some(&1) => 1..2,
            Some(&2) => 2..3,
            Some(&3) => 2..4,
            Some(_) => unreachable!(),
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

    fn denominator() -> u32 {
        4
    }
}

fn main() {
    let input = [2, 1, 1, 2, 2];

    let output = Vec::default();
    let mut bitwriter = BitWriter::endian(output, BigEndian);
    let mut encoder = Encoder::new(MyModel, 12);

    println!("encoding...");
    encoder.encode(input, &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    println!("buffer: {:?}", &buffer);

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);

    println!("\ndecoding...");
    let mut decoder = Decoder::new(MyModel, 12, bitreader).unwrap();
    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        dbg!(symbol);
    }
}
