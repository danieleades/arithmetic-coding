use arithmetic_coding::Decoder;
use arithmetic_coding::{Encoder, Model, Probability};
use bitstream_io::BigEndian;
use bitstream_io::BitWriter;
use bitstream_io::{BitReader, BitWrite};

pub enum Symbol {
    A,
    B,
    C,
}

pub struct MyModel;

impl Model for MyModel {
    type Symbol = Symbol;

    fn probability(&self, symbol: Option<&Self::Symbol>) -> Probability<u32> {
        let bounds = match symbol {
            Some(&Symbol::A) => 0..1,
            Some(&Symbol::B) => 1..2,
            Some(&Symbol::C) => 2..3,
            None => 3..4,
        };

        Probability::new(bounds, 4)
    }

    fn symbol(&self, value: u32) -> Option<Self::Symbol> {
        unimplemented!()
    }
}

fn main() {
    let input = [Symbol::A, Symbol::B, Symbol::C];

    let output = Vec::default();
    let mut bitwriter = BitWriter::endian(output, BigEndian);

    let mut encoder = Encoder::new(MyModel, 12);

    encoder.encode(input, &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    println!("buffer: {:?}", &buffer);

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);

    let _decoder = Decoder::new(MyModel, 12, bitreader).unwrap();

    dbg!(_decoder.x);
}
