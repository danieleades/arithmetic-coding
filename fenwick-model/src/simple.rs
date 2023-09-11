#![allow(missing_docs, unused)]
//! simple adaptive model using a fenwick tree

use arithmetic_coding_core::Model;

use super::Weights;
use crate::ValueError;

#[derive(Debug, Clone)]
pub struct FenwickModel<const MAX_DENOMINATOR: u64> {
    weights: Weights,
    panic_on_saturation: bool,
}

#[must_use]
pub struct Builder<const MAX_DENOMINATOR: u64> {
    model: FenwickModel<MAX_DENOMINATOR>,
}

impl<const MAX_DENOMINATOR: u64> Builder<MAX_DENOMINATOR> {
    fn new(n_symbols: usize) -> Self {
        let weights = Weights::new(n_symbols);
        let model = FenwickModel {
            weights,
            panic_on_saturation: false,
        };
        Self { model }
    }

    pub const fn panic_on_saturation(mut self) -> Self {
        self.model.panic_on_saturation = true;
        self
    }

    #[must_use]
    pub fn build(self) -> FenwickModel<MAX_DENOMINATOR> {
        self.model
    }
}

impl<const MAX_DENOMINATOR: u64> FenwickModel<MAX_DENOMINATOR> {
    pub fn builder(n_symbols: usize) -> Builder<MAX_DENOMINATOR> {
        Builder::new(n_symbols)
    }
}

impl<const MAX_DENOMINATOR: u64> Model for FenwickModel<MAX_DENOMINATOR> {
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

    fn symbol(&self, value: Self::B) -> Option<Self::Symbol> {
        self.weights.symbol(value)
    }

    fn denominator(&self) -> Self::B {
        self.weights.total()
    }

    fn update(&mut self, symbol: Option<&Self::Symbol>) {
        if self.panic_on_saturation {
            debug_assert!(
                self.denominator() < MAX_DENOMINATOR,
                "hit max denominator!"
            );
        }
        if self.denominator() < MAX_DENOMINATOR {
            self.weights.update(symbol.copied(), 1);
        }
    }

    const MAX_DENOMINATOR: Self::B = MAX_DENOMINATOR;
}
