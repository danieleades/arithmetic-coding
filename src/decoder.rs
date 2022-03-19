use std::io;

use bitstream_io::BitRead;

use crate::{Error, Model};

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
    low: u32,
    high: u32,
    input: R,
    x: u32,
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
    M::Symbol: std::fmt::Debug,
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
        let minimum_precision = frequency_bits + 2;
        debug_assert!(
            (frequency_bits + minimum_precision) <= u32::BITS,
            "not enough bits to guarantee overflow/underflow avoidance"
        );
        let precision = u32::BITS - frequency_bits;

        let low = 0;
        let high = 1 << precision;

        let x = 0;

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
            let bit = self.input.next_bit()?.unwrap_or_default();
            self.x <<= 1;
            self.x += u32::from(bit);
        }
        Ok(())
    }

    const fn half(&self) -> u32 {
        1 << (self.precision - 1)
    }

    const fn quarter(&self) -> u32 {
        1 << (self.precision - 2)
    }

    /// Read the next symbol from the stream of bits
    ///
    /// This method will return `Ok(None)` when EOF is reached.
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitRead`] cannot be read from.
    pub fn decode_symbol(&mut self) -> Result<Option<M::Symbol>, Error<M::ValueError>> {
        let range = self.high - self.low + 1;
        let value = ((self.x - self.low + 1) * self.model.denominator() - 1) / range;
        let symbol = self.model.symbol(value);

        let p = self
            .model
            .probability(symbol.as_ref())
            .map_err(Error::ValueError)?;

        self.high = self.low + (range * p.end) / self.model.denominator() - 1;
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
                self.low = 2 * (self.low - self.half());
                self.high = 2 * (self.high - self.half());
                self.x = 2 * (self.x - self.half());
            }

            match self.input.next_bit()? {
                Some(true) => {
                    self.x += 1;
                }
                Some(false) | None => (),
            }
        }

        while self.low >= self.quarter() && self.high < (3 * self.quarter()) {
            self.low = 2 * (self.low - self.quarter());
            self.high = 2 * (self.high - self.quarter());
            self.x = 2 * (self.x - self.quarter());

            match self.input.next_bit()? {
                Some(true) => {
                    self.x += 1;
                }
                Some(false) | None => (),
            }
        }

        Ok(())
    }
}
