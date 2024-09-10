use std::ops::Range;

use arithmetic_coding_core::BitStore;

#[derive(Debug)]
pub struct State<B: BitStore> {
    pub precision: u32,
    pub low: B,
    pub high: B,
}

impl<B> State<B>
where
    B: BitStore,
{
    pub fn new(precision: u32) -> Self {
        let low = B::ZERO;
        let high = B::ONE << precision;

        Self {
            precision,
            low,
            high,
        }
    }

    pub fn half(&self) -> B {
        B::ONE << (self.precision - 1)
    }

    pub fn quarter(&self) -> B {
        B::ONE << (self.precision - 2)
    }

    pub fn three_quarter(&self) -> B {
        self.half() + self.quarter()
    }

    pub fn scale(&mut self, p: Range<B>, denominator: B) {
        let range = self.high - self.low + B::ONE;

        self.high = self.low + (range * p.end) / denominator - B::ONE;
        self.low += (range * p.start) / denominator;
    }
}
