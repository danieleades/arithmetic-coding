use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

pub fn round_trip<M>(input: Vec<M::Symbol>, model: M)
where
    M: Model + Clone,
    M::Symbol: PartialEq + std::fmt::Debug + Clone,
{
    let mut bitwriter = BitWriter::endian(Vec::new(), BigEndian);

    let mut encoder = Encoder::<M>::new(model.clone());

    encoder.encode_all(input.clone(), &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);
    let mut decoder = Decoder::new(model, bitreader).unwrap();
    let mut output = Vec::new();

    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        output.push(symbol);
    }

    assert_eq!(input, output);
}
