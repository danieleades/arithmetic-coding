use std::io;

use bitstream_io::BitRead;

use crate::{BitStore, Error, Model};

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
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitRead`] cannot be read from.
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
    pub fn new(model: M, input: R) -> io::Result<Self> {
        let frequency_bits = model.max_denominator().log2() + 1;
        let precision = M::B::BITS - frequency_bits;

        Self::with_precision(model, input, precision)
    }

    /// Construct a new [`Decoder`] with a custom precision
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitRead`] cannot be read from.
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
    pub fn with_precision(model: M, input: R, precision: u32) -> io::Result<Self> {
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

        let mut encoder = Self {
            model,
            precision,
            low,
            high,
            input,
            x,
        };

        encoder.fill()?;
        Ok(encoder)
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

    /// Read the next symbol from the stream of bits
    ///
    /// This method will return `Ok(None)` when EOF is reached.
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitRead`] cannot be read from.
    pub fn decode_symbol(&mut self) -> Result<Option<M::Symbol>, Error<M::ValueError>> {
        let range = self.high - self.low + M::B::ONE;
        let value =
            ((self.x - self.low + M::B::ONE) * self.model.denominator() - M::B::ONE) / range;
        let symbol = self.model.symbol(value);

        let p = self
            .model
            .probability(symbol.as_ref())
            .map_err(Error::ValueError)?;

        self.high = self.low + (range * p.end) / self.model.denominator() - M::B::ONE;
        self.low += (range * p.start) / self.model.denominator();

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
}
