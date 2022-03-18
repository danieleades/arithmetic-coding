#![feature(exclusive_range_pattern)]
#![feature(never_type)]

mod integer {

    use std::ops::Range;

    pub struct Model;

    #[derive(Debug, thiserror::Error)]
    #[error("invalid symbol: {0}")]
    pub struct Error(u8);

    impl arithmetic_coding::Model for Model {
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

        fn max_denominator(&self) -> u32 {
            4
        }
    }
}

mod symbolic {
    use std::ops::Range;

    #[derive(Debug)]
    pub enum Symbol {
        A,
        B,
        C,
    }

    pub struct Model;

    impl arithmetic_coding::Model for Model {
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
}

fn main() {
    const PRECISION: u32 = 12;
    use arithmetic_coding::{Decoder, Encoder};
    use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};
    use symbolic::Symbol;

    let input1 = [Symbol::A, Symbol::B, Symbol::C];
    let input2 = [2, 1, 1, 2, 2];

    let mut bitwriter = BitWriter::endian(Vec::default(), BigEndian);

    println!("encoding...");

    let mut symbol_encoder = Encoder::with_precision(symbolic::Model, PRECISION);
    symbol_encoder.encode(input1, &mut bitwriter).unwrap();

    let mut integer_encoder = symbol_encoder.chain(integer::Model);
    integer_encoder.encode(input2, &mut bitwriter).unwrap();
    integer_encoder.flush(&mut bitwriter).unwrap();

    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    println!("buffer: {:?}", &buffer);

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);

    let mut symbol_decoder =
        Decoder::with_precision(symbolic::Model, bitreader, PRECISION).unwrap();

    while let Some(symbol) = symbol_decoder.decode_symbol().unwrap() {
        dbg!(symbol);
    }

    let mut integer_decoder = symbol_decoder.chain(integer::Model);

    while let Some(symbol) = integer_decoder.decode_symbol().unwrap() {
        dbg!(symbol);
    }
}
