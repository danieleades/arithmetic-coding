// these tests check the asserts that are only present in debug configurations
// so they won't pass in release mode
#![cfg(debug_assertions)]

use std::{convert::Infallible, io::Cursor, ops::Range};

use arithmetic_coding::{decoder, encoder, Decoder, Encoder};
use arithmetic_coding_core::one_shot;
use bitstream_io::{BigEndian, BitReader, BitWriter};

#[derive(Copy, Clone)]
struct SmallModel;
impl one_shot::Model for SmallModel {
    type B = u64;
    type Symbol = u64;
    type ValueError = Infallible;

    fn probability(&self, &value: &Self::Symbol) -> Result<Range<Self::B>, Self::ValueError> {
        Ok(value..value + 1)
    }

    fn max_denominator(&self) -> Self::B {
        2
    }

    fn symbol(&self, value: Self::B) -> Self::Symbol {
        value
    }
}

#[derive(Copy, Clone)]
struct BigModel;
impl one_shot::Model for BigModel {
    type B = u64;
    type Symbol = u64;
    type ValueError = Infallible;

    fn probability(&self, &value: &Self::Symbol) -> Result<Range<Self::B>, Self::ValueError> {
        Ok(value..value + 1)
    }

    fn max_denominator(&self) -> Self::B {
        u32::MAX as u64 / 2
    }

    fn symbol(&self, value: Self::B) -> Self::Symbol {
        value
    }
}

// this is one bit short of what it must be
const PRECISION: u32 = 32;

// Encoder::new should select the correct precision automagically, so we don't
// expect it to panic
#[test]
fn encoder_new_doesnt_panic() {
    Encoder::new(
        one_shot::Wrapper::new(BigModel),
        &mut BitWriter::endian(Vec::new(), BigEndian),
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn encoder_with_precision_panics() {
    Encoder::with_precision(
        one_shot::Wrapper::new(BigModel),
        &mut BitWriter::endian(Vec::new(), BigEndian),
        PRECISION,
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn encoder_with_state_panics() {
    Encoder::with_state(
        encoder::State::new(PRECISION, &mut BitWriter::endian(Vec::new(), BigEndian)),
        one_shot::Wrapper::new(BigModel),
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn encoder_chain_panics() {
    let mut writer = BitWriter::endian(Vec::new(), BigEndian);
    let encoder =
        Encoder::with_precision(one_shot::Wrapper::new(SmallModel), &mut writer, PRECISION);

    encoder.chain(one_shot::Wrapper::new(BigModel));
}

#[test]
fn decoder_new_doesnt_panic() {
    Decoder::new(
        one_shot::Wrapper::new(BigModel),
        BitReader::endian(Cursor::new(&[]), BigEndian),
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn decoder_with_precision_panics() {
    Decoder::with_precision(
        one_shot::Wrapper::new(BigModel),
        BitReader::endian(Cursor::new(&[]), BigEndian),
        PRECISION,
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn decoder_with_state_panics() {
    Decoder::with_state(
        decoder::State::new(PRECISION, BitReader::endian(Cursor::new(&[]), BigEndian)),
        one_shot::Wrapper::new(BigModel),
    );
}

#[test]
#[should_panic(expected = "not enough bits of precision to prevent overflow/underflow")]
fn decoder_chain_panics() {
    let decoder = Decoder::with_precision(
        one_shot::Wrapper::new(SmallModel),
        BitReader::endian(Cursor::new(&[]), BigEndian),
        PRECISION,
    );

    decoder.chain(one_shot::Wrapper::new(BigModel));
}
