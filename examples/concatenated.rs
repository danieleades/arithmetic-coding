#![feature(exclusive_range_pattern)]
#![feature(never_type)]

use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitRead, BitReader, BitWrite, BitWriter};

const PRECISION: u32 = 12;

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
    use symbolic::Symbol;

    let input1 = [Symbol::A, Symbol::B, Symbol::C];
    println!("input1: {:?}", input1);

    let input2 = [2, 1, 1, 2, 2];
    println!("input2: {:?}", input2);

    println!("\nencoding...");

    let buffer = encode2(symbolic::Model, &input1, integer::Model, &input2);

    println!("\nbuffer: {:?}", &buffer);

    println!("\ndecoding...");
    let (output1, output2) = decode2(symbolic::Model, integer::Model, &buffer);

    for symbol in output1 {
        println!("{:?}", symbol);
    }
    for symbol in output2 {
        println!("{:?}", symbol);
    }
}

/// Encode two sets of symbols in sequence
fn encode2<M, N>(model1: M, input1: &[M::Symbol], model2: N, input2: &[N::Symbol]) -> Vec<u8>
where
    M: Model<B = N::B>,
    N: Model,
{
    let mut bitwriter = BitWriter::endian(Vec::default(), BigEndian);

    let mut encoder1 = Encoder::with_precision(model1, PRECISION);
    encode(&mut encoder1, input1, &mut bitwriter);

    let mut encoder2 = encoder1.chain(model2);
    encode(&mut encoder2, input2, &mut bitwriter);

    encoder2.flush(&mut bitwriter).unwrap();

    bitwriter.byte_align().unwrap();
    bitwriter.into_writer()
}

/// Encode all symbols, followed by EOF. Doesn't flush the encoder (allowing
/// more bits to be concatenated)
fn encode<M, W>(encoder: &mut Encoder<M>, input: &[M::Symbol], bitwriter: &mut W)
where
    M: Model,
    W: BitWrite,
{
    for symbol in input {
        encoder.encode(Some(symbol), bitwriter).unwrap();
    }
    encoder.encode(None, bitwriter).unwrap();
}

/// Decode two sets of symbols, in sequence
fn decode2<M, N>(model1: M, model2: N, buffer: &[u8]) -> (Vec<M::Symbol>, Vec<N::Symbol>)
where
    M: Model<B = N::B>,
    N: Model,
{
    let bitreader = BitReader::endian(buffer, BigEndian);

    let mut decoder1 = Decoder::with_precision(model1, bitreader, PRECISION).unwrap();

    let output1 = decode(&mut decoder1);

    let mut decoder2 = decoder1.chain(model2);

    let output2 = decode(&mut decoder2);

    (output1, output2)
}

/// Decode all symbols from a [`Decoder`] until EOF is reached
fn decode<M, R>(decoder: &mut Decoder<M, R>) -> Vec<M::Symbol>
where
    M: Model,
    R: BitRead,
{
    let mut output = Vec::default();

    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        output.push(symbol);
    }

    output
}
