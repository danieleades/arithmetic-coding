use std::io;

use bitstream_io::BitWrite;

use crate::{BitStore, Error, Model};

// this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

/// An arithmetic encoder
///
/// An arithmetic decoder converts a stream of symbols into a stream of bits,
/// using a predictive [`Model`].
#[derive(Debug)]
pub struct Encoder<'a, M, W>
where
    M: Model,
    W: BitWrite,
{
    model: M,
    precision: u32,
    low: M::B,
    high: M::B,
    pending: usize,
    output: &'a mut W,
}

impl<'a, M, W> Encoder<'a, M, W>
where
    M: Model,
    W: BitWrite,
{
    /// Construct a new [`Encoder`].
    ///
    /// The 'precision' of the encoder is maximised, based on the number of bits
    /// needed to represent the [`Model::denominator`]. 'precision' bits is
    /// equal to [`BitStore::BITS`] - [`Model::denominator`] bits. If you need
    /// to set the precision manually, use [`Encoder::with_precision`].
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
    pub fn new(model: M, bitwriter: &'a mut W) -> Self {
        let frequency_bits = model.max_denominator().log2() + 1;
        let precision = M::B::BITS - frequency_bits;
        Self::with_precision(model, bitwriter, precision)
    }

    /// Construct a new [`Encoder`] with a custom precision.
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
    pub fn with_precision(model: M, bitwriter: &'a mut W, precision: u32) -> Self {
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
        let pending = 0;

        Self {
            model,
            precision,
            low,
            high,
            pending,
            output: bitwriter,
        }
    }

    /// Encode a stream of symbols into the provided output.
    ///
    /// This method will encode all the symbols in the iterator, followed by EOF
    /// (`None`), and then call [`Encoder::flush`].
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitWrite`] cannot be written
    /// to.
    pub fn encode_all(
        &mut self,
        symbols: impl IntoIterator<Item = M::Symbol>,
    ) -> Result<(), Error<M::ValueError>> {
        for symbol in symbols {
            self.encode(Some(&symbol))?;
        }
        self.encode(None)?;
        self.flush()?;
        Ok(())
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

    /// Encode a symbol into the provided output.
    ///
    /// When you finish encoding symbols, you must manually encode an EOF symbol
    /// by calling [`Encoder::encode`] with `None`.
    ///
    /// The internal buffer must be manually flushed using [`Encoder::flush`].
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitWrite`] cannot be written
    /// to.
    pub fn encode(&mut self, symbol: Option<&M::Symbol>) -> Result<(), Error<M::ValueError>> {
        let range = self.high - self.low + M::B::ONE;
        let p = self.model.probability(symbol).map_err(Error::ValueError)?;
        let denominator = self.model.denominator();
        debug_assert!(
            denominator <= self.model.max_denominator(),
            "denominator is greater than maximum!"
        );

        self.high = self.low + (range * p.end) / denominator - M::B::ONE;
        self.low += (range * p.start) / denominator;

        self.model.update(symbol);
        self.normalise()?;

        Ok(())
    }

    fn normalise(&mut self) -> io::Result<()> {
        while self.high < self.half() || self.low >= self.half() {
            if self.high < self.half() {
                self.emit(false)?;
                self.high <<= 1;
                self.low <<= 1;
            } else {
                // self.low >= self.half()
                self.emit(true)?;
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

    fn emit(&mut self, bit: bool) -> io::Result<()> {
        self.output.write_bit(bit)?;
        for _ in 0..self.pending {
            self.output.write_bit(!bit)?;
        }
        self.pending = 0;
        Ok(())
    }

    /// Flush any pending bits from the buffer
    ///
    /// This method must be called when you finish writing symbols to a stream
    /// of bits. This is called automatically when you use
    /// [`Encoder::encode_all`].
    ///
    /// # Errors
    ///
    /// This method can fail if the underlying [`BitWrite`] cannot be written
    /// to.
    pub fn flush(&mut self) -> io::Result<()> {
        self.pending += 1;
        if self.low <= self.quarter() {
            self.emit(false)?;
        } else {
            self.emit(true)?;
        }

        Ok(())
    }

    /// Reuse the internal state of the Encoder with a new model.
    ///
    /// Allows for chaining multiple sequences of symbols into a single stream
    /// of bits
    pub fn chain<X>(self, model: X) -> Encoder<'a, X, W>
    where
        X: Model<B = M::B>,
    {
        Encoder {
            model,
            precision: self.precision,
            low: self.low,
            high: self.high,
            pending: self.pending,
            output: self.output,
        }
    }
}
