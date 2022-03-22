#![allow(missing_docs, unused)]
//! simple adaptive model using a fenwick tree

use arithmetic_coding::Model;

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
            max_denominator: 1 << 20,
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

#[cfg(test)]
mod tests {
    use arithmetic_coding::Model;

    use super::FenwickModel;

    #[test]
    fn denominator() {
        let fenwick_model: FenwickModel = FenwickModel::with_symbols(4);
        assert_eq!(fenwick_model.denominator(), 5);
    }

    #[test]
    fn probability() {
        let fenwick_model: FenwickModel = FenwickModel::with_symbols(4);
        assert_eq!(fenwick_model.probability(None).unwrap(), 0..1);
        assert_eq!(fenwick_model.probability(Some(&0)).unwrap(), 1..2);
        assert_eq!(fenwick_model.probability(Some(&1)).unwrap(), 2..3);
        assert_eq!(fenwick_model.probability(Some(&2)).unwrap(), 3..4);
        assert_eq!(fenwick_model.probability(Some(&3)).unwrap(), 4..5);
    }

    #[test]
    #[should_panic]
    fn probability_out_of_bounds() {
        let fenwick_model: FenwickModel = FenwickModel::with_symbols(4);
        fenwick_model.probability(Some(&4)).unwrap();
    }

    #[test]
    fn symbol() {
        let mut fenwick_model: FenwickModel = FenwickModel::with_symbols(4);
        assert_eq!(fenwick_model.symbol(0), None);
        assert_eq!(fenwick_model.symbol(1), Some(0));
        assert_eq!(fenwick_model.symbol(2), Some(1));
        assert_eq!(fenwick_model.symbol(3), Some(2));
        assert_eq!(fenwick_model.symbol(4), Some(3));

        fenwick_model.update(Some(&0));

        assert_eq!(fenwick_model.symbol(3), Some(1));
    }

    #[test]
    #[should_panic]
    fn symbol_out_of_bounds() {
        let fenwick_model: FenwickModel = FenwickModel::with_symbols(4);
        fenwick_model.symbol(5);
    }
}
