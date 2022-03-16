use std::io;

use crate::Model;
use bitstream_io::BitRead;

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
    pub fn new(model: M, precision: u32, mut input: R) -> io::Result<Self> {
        let low = 0;
        let high = 2u32.pow(precision);

        let mut x = 0;

        for i in 1..precision {
            match input.next_bit()? {
                Some(true) => {
                    x += 2u32.pow(precision - i);
                }
                Some(false) => (),
                None => break,
            }
        }

        Ok(Self {
            model,
            precision,
            low,
            high,
            input,
            x,
        })
    }

    const fn half(&self) -> u32 {
        2u32.pow(self.precision - 1)
    }

    const fn quarter(&self) -> u32 {
        2u32.pow(self.precision - 2)
    }

    pub fn decode_symbol(&mut self) -> io::Result<Option<M::Symbol>> {
        let range = self.high - self.low + 1;
        let value = ((self.x - self.low + 1) * M::denominator() - 1) / range;
        let symbol = self.model.symbol(value);

        let p = self.model.probability(symbol.as_ref());

        self.high = self.low + (range * p.end) / M::denominator() - 1;
        self.low += (range * p.start) / M::denominator();

        self.normalise()?;
        Ok(symbol)
    }

    fn normalise(&mut self) -> io::Result<()> {
        // this algorithm is derived from this article - https://marknelson.us/posts/2014/10/19/data-compression-with-arithmetic-coding.html

        while self.high < self.half() || self.low >= self.half() {
            if self.high < self.half() {
                self.high *= 2;
                self.low *= 2;
                self.x *= 2;
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

        while self.quarter() <= self.low && self.high < 3 * self.quarter() {
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
