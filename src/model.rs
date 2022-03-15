use crate::Probability;

pub trait Model<T = u32> {
    type Symbol;
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Probability<T>;
    fn symbol(&self, value: T) -> Option<Self::Symbol>;
}
