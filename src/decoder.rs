//! The [`Decoder`] half of the arithmetic coding library.

use std::{io, ops::Range};

use bitstream_io::BitRead;

use crate::{
    common::{self, assert_precision_sufficient},
    BitStore, Model,
};

// this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

/// An arithmetic decoder
///
/// An arithmetic decoder converts a stream of bytes into a stream of some
/// output symbol, using a predictive [`Model`].
#[derive(Debug)]
pub struct Decoder<M, R>
where
    M: Model,
    R: BitRead,
{
    model: M,
    state: State<M::B, R>,
}

trait BitReadExt {
    fn next_bit(&mut self) -> io::Result<Option<bool>>;
}

impl<R: BitRead> BitReadExt for R {
    fn next_bit(&mut self) -> io::Result<Option<bool>> {
        match self.read_bit() {
            Ok(bit) => Ok(Some(bit)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<M, R> Decoder<M, R>
where
    M: Model,
    R: BitRead,
{
    /// Construct a new [`Decoder`]
    ///
    /// The 'precision' of the encoder is maximised, based on the number of bits
    /// needed to represent the [`Model::denominator`]. 'precision' bits is
    /// equal to [`u32::BITS`] - [`Model::denominator`] bits.
    ///
    /// # Panics
    ///
    /// The calculation of the number of bits used for 'precision' is subject to
    /// the following constraints:
    ///
    /// - The total available bits is [`u32::BITS`]
    /// - The precision must use at least 2 more bits than that needed to
    ///   represent [`Model::denominator`]
    ///
    /// If these constraints cannot be satisfied this method will panic in debug
    /// builds
    pub fn new(model: M, input: R) -> Self {
        let frequency_bits = model.max_denominator().log2() + 1;
        let precision = M::B::BITS - frequency_bits;

        Self::with_precision(model, input, precision)
    }

    /// Construct a new [`Decoder`] with a custom precision
    ///
    /// # Panics
    ///
    /// The calculation of the number of bits used for 'precision' is subject to
    /// the following constraints:
    ///
    /// - The total available bits is [`BitStore::BITS`]
    /// - The precision must use at least 2 more bits than that needed to
    ///   represent [`Model::denominator`]
    ///
    /// If these constraints cannot be satisfied this method will panic in debug
    /// builds
    pub fn with_precision(model: M, input: R, precision: u32) -> Self {
        let state = State::new(precision, input);
        Self::with_state(state, model)
    }

    /// todo
    pub fn with_state(state: State<M::B, R>, model: M) -> Self {
        #[cfg(debug_assertions)]
        assert_precision_sufficient::<M>(model.max_denominator(), state.state.precision);

        Self { model, state }
    }

    /// Return an iterator over the decoded symbols.
    ///
    /// The iterator will continue returning symbols until EOF is reached
    pub fn decode_all(&mut self) -> DecodeIter<M, R> {
        DecodeIter { decoder: self }
    }

    /// Read the next symbol from the stream of bits
    ///
    /// This method will return `Ok(None)` when EOF is reached.
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitRead`] cannot be read from.
    #[allow(clippy::missing_panics_doc)]
    pub fn decode(&mut self) -> io::Result<Option<M::Symbol>> {
        self.state.initialise()?;

        let denominator = self.model.denominator();
        debug_assert!(
            denominator <= self.model.max_denominator(),
            "denominator is greater than maximum!"
        );
        let value = self.state.value(denominator);
        let symbol = self.model.symbol(value);

        let p = self
            .model
            .probability(symbol.as_ref())
            .expect("this should not be able to fail. Check the implementation of the model.");

        self.state.scale(p, denominator)?;
        self.model.update(symbol.as_ref());

        Ok(symbol)
    }

    /// Reuse the internal state of the Decoder with a new model.
    ///
    /// Allows for chaining multiple sequences of symbols from a single stream
    /// of bits
    pub fn chain<X>(self, model: X) -> Decoder<X, R>
    where
        X: Model<B = M::B>,
    {
        Decoder::with_state(self.state, model)
    }

    /// todo
    pub fn into_inner(self) -> (M, State<M::B, R>) {
        (self.model, self.state)
    }
}

/// The iterator returned by the [`Model::decode_all`] method
#[allow(missing_debug_implementations)]
pub struct DecodeIter<'a, M, R>
where
    M: Model,
    R: BitRead,
{
    decoder: &'a mut Decoder<M, R>,
}

impl<M, R> Iterator for DecodeIter<'_, M, R>
where
    M: Model,
    R: BitRead,
{
    type Item = io::Result<M::Symbol>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.decode().transpose()
    }
}

/// A convenience struct which stores the internal state of an [`Decoder`].
#[derive(Debug)]
pub struct State<B, R>
where
    B: BitStore,
    R: BitRead,
{
    state: common::State<B>,
    input: R,
    x: B,
    uninitialised: bool,
}

impl<B, R> State<B, R>
where
    B: BitStore,
    R: BitRead,
{
    /// todo
    pub fn new(precision: u32, input: R) -> Self {
        let state = common::State::new(precision);
        let x = B::ZERO;

        Self {
            state,
            input,
            x,
            uninitialised: true,
        }
    }

    fn normalise(&mut self) -> io::Result<()> {
        while self.state.high < self.state.half() || self.state.low >= self.state.half() {
            if self.state.high < self.state.half() {
                self.state.high <<= 1;
                self.state.low <<= 1;
                self.x <<= 1;
            } else {
                // self.low >= self.half()
                self.state.low = (self.state.low - self.state.half()) << 1;
                self.state.high = (self.state.high - self.state.half()) << 1;
                self.x = (self.x - self.state.half()) << 1;
            }

            if self.input.next_bit()? == Some(true) {
                self.x += B::ONE;
            }
        }

        while self.state.low >= self.state.quarter()
            && self.state.high < (self.state.three_quarter())
        {
            self.state.low = (self.state.low - self.state.quarter()) << 1;
            self.state.high = (self.state.high - self.state.quarter()) << 1;
            self.x = (self.x - self.state.quarter()) << 1;

            if self.input.next_bit()? == Some(true) {
                self.x += B::ONE;
            }
        }

        Ok(())
    }

    fn scale(&mut self, p: Range<B>, denominator: B) -> io::Result<()> {
        self.state.scale(p, denominator);
        self.normalise()
    }

    fn value(&self, denominator: B) -> B {
        let range = self.state.high - self.state.low + B::ONE;
        ((self.x - self.state.low + B::ONE) * denominator - B::ONE) / range
    }

    fn fill(&mut self) -> io::Result<()> {
        for _ in 0..self.state.precision {
            self.x <<= 1;
            if self.input.next_bit()? == Some(true) {
                self.x += B::ONE;
            }
        }
        Ok(())
    }

    fn initialise(&mut self) -> io::Result<()> {
        if self.uninitialised {
            self.fill()?;
            self.uninitialised = false;
        }
        Ok(())
    }
}
