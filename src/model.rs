use std::ops::Range;

pub trait Model {
    type Symbol;
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Range<u32>;
    fn denominator() -> u32;
    fn symbol(&self, value: u32) -> Option<Self::Symbol>;
    fn update(&mut self, _symbol: Option<&Self::Symbol>) {}
}
