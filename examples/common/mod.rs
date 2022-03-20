use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

#[allow(unused)]
pub fn round_trip<M>(model: M, input: Vec<M::Symbol>)
where
    M: Model + Clone,
    M::Symbol: std::fmt::Debug,
{
    println!("input: {:?}", &input);

    println!("\nencoding...");
    let buffer = encode(model.clone(), input);

    println!("buffer: {:?}", &buffer);

    println!("\ndecoding...");
    for symbol in decode(model, &buffer) {
        println!("{:?}", symbol);
    }
}

pub fn encode<M, I>(model: M, input: I) -> Vec<u8>
where
    M: Model,
    I: IntoIterator<Item = M::Symbol>,
{
    let mut bitwriter = BitWriter::endian(Vec::new(), BigEndian);
    let mut encoder = Encoder::<M>::new(model);

    encoder.encode_all(input, &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    bitwriter.into_writer()
}

pub fn decode<M>(model: M, buffer: &[u8]) -> Vec<M::Symbol>
where
    M: Model,
{
    let bitreader = BitReader::endian(buffer, BigEndian);
    let mut decoder = Decoder::new(model, bitreader).unwrap();
    let mut output = Vec::new();

    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        output.push(symbol);
    }
    output
}
