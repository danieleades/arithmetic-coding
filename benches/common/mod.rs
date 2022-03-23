use arithmetic_coding::{Decoder, Encoder, Model};
use bitstream_io::{BigEndian, BitReader, BitWrite, BitWriter};

pub fn round_trip<M>(model: M, input: &[M::Symbol])
where
    M: Model + Clone,
    M::Symbol: Copy + std::fmt::Debug + PartialEq,
{
    let buffer = encode(model.clone(), input.iter().copied());

    let mut output = Vec::with_capacity(input.len());
    for symbol in decode(model, &buffer) {
        output.push(symbol);
    }

    assert_eq!(input, output.as_slice());
}

pub fn encode<M, I>(model: M, input: I) -> Vec<u8>
where
    M: Model,
    I: IntoIterator<Item = M::Symbol>,
{
    let mut bitwriter = BitWriter::endian(Vec::new(), BigEndian);
    let mut encoder = Encoder::new(model, &mut bitwriter);

    encoder.encode_all(input).unwrap();
    bitwriter.byte_align().unwrap();

    bitwriter.into_writer()
}

pub fn decode<M>(model: M, buffer: &[u8]) -> Vec<M::Symbol>
where
    M: Model,
{
    let bitreader = BitReader::endian(buffer, BigEndian);
    let mut decoder = Decoder::new(model, bitreader);
    decoder.decode_all().map(Result::unwrap).collect()
}
