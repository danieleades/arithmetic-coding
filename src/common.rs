use std::ops::Range;

use arithmetic_coding_core::BitStore;

use crate::Model;

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
        let high = (B::ONE << precision) - B::ONE;

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

pub fn assert_precision_sufficient<M: Model>(max_denominator: M::B, precision: u32) {
    let frequency_bits = max_denominator.log2() + 1;
    assert!(
        (precision >= (frequency_bits + 2)),
        "not enough bits of precision to prevent overflow/underflow",
    );
    assert!(
        (frequency_bits + precision) <= M::B::BITS,
        "not enough bits in BitStore to support the required precision",
    );
}
