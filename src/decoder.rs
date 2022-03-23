use std::io;

use bitstream_io::BitRead;

use crate::{BitStore, Model};

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
    precision: u32,
    low: M::B,
    high: M::B,
    input: R,
    x: M::B,
    uninitialised: bool,
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
        let frequency_bits = model.max_denominator().log2() + 1;
        debug_assert!(
            (precision >= (frequency_bits + 2)),
            "not enough bits of precision to prevent overflow/underflow",
        );
        debug_assert!(
            (frequency_bits + precision) <= M::B::BITS,
            "not enough bits in BitStore to support the required precision",
        );

        let low = M::B::ZERO;
        let high = M::B::ONE << precision;
        let x = M::B::ZERO;

        Self {
            model,
            precision,
            low,
            high,
            input,
            x,
            uninitialised: true,
        }
    }

    fn fill(&mut self) -> io::Result<()> {
        for _ in 0..self.precision {
            self.x <<= 1;
            match self.input.next_bit()? {
                Some(true) => {
                    self.x += M::B::ONE;
                }
                Some(false) | None => (),
            }
        }
        Ok(())
    }

    fn half(&self) -> M::B {
        M::B::ONE << (self.precision - 1)
    }

    fn quarter(&self) -> M::B {
        M::B::ONE << (self.precision - 2)
    }

    fn three_quarter(&self) -> M::B {
        self.half() + self.quarter()
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
    pub fn decode(&mut self) -> io::Result<Option<M::Symbol>> {
        if self.uninitialised {
            self.fill()?;
            self.uninitialised = false;
        }

        let range = self.high - self.low + M::B::ONE;
        let denominator = self.model.denominator();
        debug_assert!(
            denominator <= self.model.max_denominator(),
            "denominator is greater than maximum!"
        );
        let value = ((self.x - self.low + M::B::ONE) * denominator - M::B::ONE) / range;
        let symbol = self.model.symbol(value);

        let p = self
            .model
            .probability(symbol.as_ref())
            .expect("this should not be able to fail. Check the implementation of the model.");

        self.high = self.low + (range * p.end) / denominator - M::B::ONE;
        self.low += (range * p.start) / denominator;

        self.model.update(symbol.as_ref());
        self.normalise()?;

        Ok(symbol)
    }

    fn normalise(&mut self) -> io::Result<()> {
        while self.high < self.half() || self.low >= self.half() {
            if self.high < self.half() {
                self.high <<= 1;
                self.low <<= 1;
                self.x <<= 1;
            } else {
                // self.low >= self.half()
                self.low = (self.low - self.half()) << 1;
                self.high = (self.high - self.half()) << 1;
                self.x = (self.x - self.half()) << 1;
            }

            match self.input.next_bit()? {
                Some(true) => {
                    self.x += M::B::ONE;
                }
                Some(false) | None => (),
            }
        }

        while self.low >= self.quarter() && self.high < (self.three_quarter()) {
            self.low = (self.low - self.quarter()) << 1;
            self.high = (self.high - self.quarter()) << 1;
            self.x = (self.x - self.quarter()) << 1;

            match self.input.next_bit()? {
                Some(true) => {
                    self.x += M::B::ONE;
                }
                Some(false) | None => (),
            }
        }

        Ok(())
    }

    /// Reuse the internal state of the Decoder with a new model.
    ///
    /// Allows for chaining multiple sequences of symbols from a single stream
    /// of bits
    pub fn chain<X>(self, model: X) -> Decoder<X, R>
    where
        X: Model<B = M::B>,
    {
        Decoder {
            model,
            precision: self.precision,
            low: self.low,
            high: self.high,
            input: self.input,
            x: self.x,
            uninitialised: self.uninitialised,
        }
    }
}

/// The iterator returned by the [`Model::decode_all`] method
#[derive(Debug)]
pub struct DecodeIter<'a, M, R>
where
    M: Model,
    R: BitRead,
{
    decoder: &'a mut Decoder<M, R>,
}

impl<'a, M, R> Iterator for DecodeIter<'a, M, R>
where
    M: Model,
    R: BitRead,
{
    type Item = io::Result<M::Symbol>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.decode().transpose()
    }
}
