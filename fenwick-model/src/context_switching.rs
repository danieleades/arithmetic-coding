#![allow(missing_docs, unused)]
//! Fenwick tree based context-switching model

use arithmetic_coding_core::Model;

use super::Weights;

#[derive(Debug, Clone)]
pub struct FenwickModel {
    contexts: Vec<Weights>,
    previous_context: usize,
    current_context: usize,
    denominator: u64,
    max_denominator: u64,
}

impl FenwickModel {
    #[must_use]
    pub fn with_symbols(symbols: usize) -> Self {
        let mut contexts = Vec::with_capacity(symbols + 1);
        let mut denominator = 0;
        let max_denominator = 1 << 17;

        for _ in 0..=symbols {
            let weight = Weights::new(symbols);
            denominator = denominator.max(weight.total());
            contexts.push(Weights::new(symbols));
        }

        Self {
            contexts,
            previous_context: 1,
            current_context: 1,
            denominator,
            max_denominator,
        }
    }

    fn context(&self) -> &Weights {
        &self.contexts[self.current_context]
    }

    fn context_mut(&mut self) -> &mut Weights {
        &mut self.contexts[self.current_context]
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid symbol received: {0}")]
pub struct ValueError(usize);

impl Model for FenwickModel {
    type B = u64;
    type Symbol = usize;
    type ValueError = ValueError;

    fn probability(
        &self,
        symbol: Option<&Self::Symbol>,
    ) -> Result<std::ops::Range<Self::B>, Self::ValueError> {
        Ok(self.context().range(symbol.copied()))
    }

    fn max_denominator(&self) -> Self::B {
        self.max_denominator
    }

    fn symbol(&self, value: Self::B) -> Option<Self::Symbol> {
        self.context().symbol(value)
    }

    fn denominator(&self) -> Self::B {
        self.context().total
    }

    fn update(&mut self, symbol: Option<&Self::Symbol>) {
        debug_assert!(
            self.denominator() < self.max_denominator,
            "hit max denominator!"
        );
        if self.denominator() < self.max_denominator {
            self.context_mut().update(symbol.copied(), 1);
            self.denominator = self.denominator.max(self.context().total());
        }
        self.current_context = symbol.map(|x| x + 1).unwrap_or_default();
    }
}
