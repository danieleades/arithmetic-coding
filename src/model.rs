use std::ops::Range;

/// A [`Model`] is used to calculate the probability of a given symbol occuring
/// in a sequence. The [`Model`] is used both for encoding and decoding.
///
/// The more accurately a [`Model`] is able to predict the next symbol, the
/// greater the compression ratio will be.
///
/// # Example
///
/// ```
/// use arithmetic_coding::Model;
///
/// pub enum Symbol {
///     A,
///     B,
///     C,
/// }
///
/// pub struct MyModel;
///
/// impl Model for MyModel {
///     type Symbol = Symbol;
///
///     fn probability(&self, symbol: Option<&Self::Symbol>) -> Range<u32> {
///         match symbol {
///             None => 0..1,
///             Some(&Symbol::A) => 1..2,
///             Some(&Symbol::B) => 2..3,
///             Some(&Symbol::C) => 3..4,
///         }
///     }
///
///     fn symbol(&self, value: u32) -> Option<Self::Symbol> {
///         match value {
///             0..1 => None,
///             1..2 => Some(Symbol::A),
///             2..3 => Some(Symbol::B),
///             3..4 => Some(Symbol::C),
///             _ => unreachable!(),
///         }
///     }
///
///     fn denominator(&self) -> u32 {
///         4
///     }
/// }
/// ```
pub trait Model {
    /// The type of symbol this [`Model`] describes
    type Symbol;

    /// Given a symbol, return an interval representing the probability of that
    /// symbol occurring.
    ///
    /// This is given as a range, over the denominator given by
    /// [`Model::denominator`]. This range should in general include `EOF`,
    /// which is denoted by `None`.
    ///
    /// For example, from the set {heads, tails}, the interval representing
    /// heads could be `0..1`, and tails would be `1..2`, and `EOF` could be
    /// `2..3` (with a denominator of `3`).
    ///
    /// This is the inverse of the [`Model::symbol`] method
    fn probability(&self, symbol: Option<&Self::Symbol>) -> Range<u32>;

    /// The denominator for probability ranges. See [`Model::probability`].
    ///
    /// This value is also used to calculate an appropriate precision for the
    /// encoding.
    fn denominator(&self) -> u32;

    /// Given a value, return the symbol whose probability range it falls in.
    ///
    /// `None` indicates `EOF`
    ///
    /// This is the inverse of the [`Model::range`] method
    fn symbol(&self, value: u32) -> Option<Self::Symbol>;

    /// Update the current state of the model with the latest symbol.
    ///
    /// This method only needs to be implemented for 'adaptive' models. It's a
    /// no-op by default.
    fn update(&mut self, _symbol: Option<&Self::Symbol>) {}
}
