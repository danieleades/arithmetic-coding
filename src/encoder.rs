use std::io;

use bitstream_io::BitWrite;

use crate::{util, BitStore, Error, Model};

// this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

/// An arithmetic encoder
///
/// An arithmetic decoder converts a stream of symbols into a stream of bits,
/// using a predictive [`Model`].
#[derive(Debug)]
pub struct Encoder<M, B = u32>
where
    B: BitStore,
    M: Model,
{
    model: M,
    precision: u32,
    low: B,
    high: B,
    pending: usize,
}

impl<M> Encoder<M, u32>
where
    M: Model,
{
    /// Construct a new [`Encoder`],using the default [`BitStore`] (`u32`).
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
    pub fn new(model: M) -> Self {
        Encoder::with_bitstore(model)
    }
}

impl<M, B> Encoder<M, B>
where
    B: BitStore,
    M: Model,
{
    /// Construct a new [`Encoder`]
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
    pub fn with_bitstore(model: M) -> Self {
        let precision = util::precision(model.max_denominator());

        let low = B::ZERO;
        let high = B::ONE << precision;
        let pending = 0;

        Self {
            model,
            precision,
            low,
            high,
            pending,
        }
    }

    fn three_quarter(&self) -> B {
        self.half() + self.quarter()
    }

    fn half(&self) -> B {
        B::ONE << (self.precision - 1)
    }

    fn quarter(&self) -> B {
        B::ONE << (self.precision - 2)
    }

    fn encode_symbol<W: BitWrite>(
        &mut self,
        symbol: Option<&M::Symbol>,
        output: &mut W,
    ) -> Result<(), Error<M::ValueError>> {
        let range = self.high - self.low + B::ONE;
        let p = self.model.probability(symbol).map_err(Error::ValueError)?;

        self.high =
            self.low + (range * B::from(p.end)) / B::from(self.model.denominator()) - B::ONE;
        self.low += (range * B::from(p.start)) / B::from(self.model.denominator());
        self.model.update(symbol);
        self.normalise(output)?;
        Ok(())
    }

    fn normalise<W: BitWrite>(&mut self, output: &mut W) -> io::Result<()> {
        while self.high < self.half() || self.low >= self.half() {
            if self.high < self.half() {
                self.emit(false, output)?;
                self.high <<= 1;
                self.low <<= 1;
            } else {
                // self.low >= self.half()
                self.emit(true, output)?;
                self.low = (self.low - self.half()) << 1;
                self.high = (self.high - self.half()) << 1;
            }
        }

        while self.low >= self.quarter() && self.high < (self.three_quarter()) {
            self.pending += 1;
            self.low = (self.low - self.quarter()) << 1;
            self.high = (self.high - self.quarter()) << 1;
        }

        Ok(())
    }

    fn emit<W: BitWrite>(&mut self, bit: bool, output: &mut W) -> io::Result<()> {
        output.write_bit(bit)?;
        for _ in 0..self.pending {
            output.write_bit(!bit)?;
        }
        self.pending = 0;
        Ok(())
    }

    fn flush<W: BitWrite>(&mut self, output: &mut W) -> io::Result<()> {
        self.pending += 1;
        if self.low <= self.quarter() {
            self.emit(false, output)?;
        } else {
            self.emit(true, output)?;
        }

        Ok(())
    }

    /// Encode a stream of symbols into the provided output
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitWrite`] cannot be written
    /// to.
    pub fn encode<W: BitWrite>(
        &mut self,
        symbols: impl IntoIterator<Item = M::Symbol>,
        output: &mut W,
    ) -> Result<(), Error<M::ValueError>> {
        for symbol in symbols {
            self.encode_symbol(Some(&symbol), output)?;
        }
        self.encode_symbol(None, output)?;
        self.flush(output)?;
        Ok(())
    }
}
