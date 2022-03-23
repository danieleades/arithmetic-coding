#![allow(missing_docs, unused)]
//! simple adaptive model using a fenwick tree

use arithmetic_coding_core::Model;

use super::Weights;

#[derive(Debug, Clone)]
pub struct FenwickModel {
    weights: Weights,
    max_denominator: u64,
}

impl FenwickModel {
    #[must_use]
    pub fn with_symbols(symbols: usize) -> Self {
        let weights = Weights::new(symbols);

        Self {
            weights,
            max_denominator: 1 << 17,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid symbol received: {0}")]
pub struct ValueError(pub usize);

impl Model for FenwickModel {
    type B = u64;
    type Symbol = usize;
    type ValueError = ValueError;

    fn probability(
        &self,
        symbol: Option<&Self::Symbol>,
    ) -> Result<std::ops::Range<Self::B>, Self::ValueError> {
        if let Some(s) = symbol.copied() {
            if s >= self.weights.len() {
                Err(ValueError(s))
            } else {
                Ok(self.weights.range(Some(s)))
            }
        } else {
            Ok(self.weights.range(None))
        }
    }

    fn max_denominator(&self) -> Self::B {
        self.max_denominator
    }

    fn symbol(&self, value: Self::B) -> Option<Self::Symbol> {
        self.weights.symbol(value)
    }

    fn denominator(&self) -> Self::B {
        self.weights.total()
    }

    fn update(&mut self, symbol: Option<&Self::Symbol>) {
        debug_assert!(
            self.denominator() < self.max_denominator,
            "hit max denominator!"
        );
        if self.denominator() < self.max_denominator {
            self.weights.update(symbol.copied(), 1);
        }
    }
}
