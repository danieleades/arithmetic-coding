use arithmetic_coding::{BitStore, Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

pub fn round_trip<B, M>(input: Vec<M::Symbol>, model: M)
where
    B: BitStore,
    M: Model + Clone,
    M::Symbol: PartialEq + std::fmt::Debug + Clone,
{
    let mut bitwriter = BitWriter::endian(Vec::new(), BigEndian);

    let mut encoder = Encoder::<M, B>::with_bitstore(model.clone());

    encoder.encode(input.clone(), &mut bitwriter).unwrap();
    bitwriter.byte_align().unwrap();

    let buffer = bitwriter.into_writer();

    let bitreader = BitReader::endian(buffer.as_slice(), BigEndian);
    let mut decoder = Decoder::new(model, bitreader).unwrap();
    let mut output = Vec::new();

    while let Some(symbol) = decoder.decode_symbol().unwrap() {
        output.push(symbol);
    }

    // assert_eq!(input, output);
    panic!()
}
