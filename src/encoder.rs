use std::io;

use bitstream_io::BitWrite;

use crate::{util, BitStore, Error, Model};

// this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

/// An arithmetic encoder
///
/// An arithmetic decoder converts a stream of symbols into a stream of bits,
/// using a predictive [`Model`].
#[derive(Debug)]
pub struct Encoder<M>
where
    M: Model,
{
    model: M,
    precision: u32,
    low: M::B,
    high: M::B,
    pending: usize,
}

impl<M> Encoder<M>
where
    M: Model,
{
    /// Construct a new [`Encoder`] with a custom [`BitStore`].
    ///
    /// The 'precision' of the encoder is maximised, based on the number of bits
    /// needed to represent the [`Model::denominator`]. 'precision' bits is
    /// equal to [`BitStore::BITS`] - [`Model::denominator`] bits.
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
    pub fn new(model: M) -> Self {
        let precision = util::precision::<M::B>(model.max_denominator());

        let low = M::B::ZERO;
        let high = M::B::ONE << precision;
        let pending = 0;

        Self {
            model,
            precision,
            low,
            high,
            pending,
        }
    }

    fn three_quarter(&self) -> M::B {
        self.half() + self.quarter()
    }

    fn half(&self) -> M::B {
        M::B::ONE << (self.precision - 1)
    }

    fn quarter(&self) -> M::B {
        M::B::ONE << (self.precision - 2)
    }

    fn encode_symbol<W: BitWrite>(
        &mut self,
        symbol: Option<&M::Symbol>,
        output: &mut W,
    ) -> Result<(), Error<M::ValueError>> {
        let range = self.high - self.low + M::B::ONE;
        let p = self.model.probability(symbol).map_err(Error::ValueError)?;

        self.high = self.low + (range * p.end) / self.model.denominator() - M::B::ONE;
        self.low += (range * p.start) / self.model.denominator();
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
