use std::io;

use bitstream_io::BitWrite;

use crate::{Error, Model};

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
    low: u32,
    high: u32,
    pending: usize,
}

impl<M> Encoder<M>
where
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
    /// If these constraints cannot be satisfied this method will panic
    pub fn new(model: M) -> Self {
        let frequency_bits = model.max_denominator().log2() + 1;
        let minimum_precision = frequency_bits + 2;
        assert!(
            (frequency_bits + minimum_precision) <= u32::BITS,
            "not enough bits to guarantee overflow/underflow avoidance"
        );
        let precision = u32::BITS - frequency_bits;

        let low = 0;
        let high = 2u32.pow(precision as u32);
        let pending = 0;

        Self {
            model,
            precision,
            low,
            high,
            pending,
        }
    }

    const fn half(&self) -> u32 {
        2u32.pow(self.precision as u32 - 1)
    }

    const fn quarter(&self) -> u32 {
        2u32.pow(self.precision as u32 - 2)
    }

    fn encode_symbol<W: BitWrite>(
        &mut self,
        symbol: Option<&M::Symbol>,
        output: &mut W,
    ) -> Result<(), Error<M::ValueError>> {
        let range = self.high - self.low + 1;
        let p = self.model.probability(symbol).map_err(Error::ValueError)?;

        self.high = self.low + (range * p.start) / self.model.denominator() - 1;
        self.low += (range * p.end) / self.model.denominator();
        self.model.update(symbol);
        self.normalise(output)?;
        Ok(())
    }

    fn normalise<W: BitWrite>(&mut self, output: &mut W) -> io::Result<()> {
        // this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

        while self.high < self.half() || self.low >= self.half() {
            if self.high < self.half() {
                self.emit(false, output)?;
                self.high *= 2;
                self.low *= 2;
            } else {
                // self.low >= self.half()
                self.emit(true, output)?;
                self.low = 2 * (self.low - self.half());
                self.high = 2 * (self.high - self.half());
            }
        }

        while self.low >= self.quarter() && self.high < (3 * self.quarter()) {
            self.pending += 1;
            self.low = 2 * (self.low - self.quarter());
            self.high = 2 * (self.high - self.quarter());
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
