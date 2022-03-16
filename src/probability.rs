use std::ops::Range;

pub struct Probability<T> {
    bounds: Range<T>,
    denominator: T,
}

impl<T> Probability<T>
where
    T: PartialOrd,
{
    pub const fn new(bounds: Range<T>, denominator: T) -> Self {
        Self {
            bounds,
            denominator,
        }
    }

    pub const fn low(&self) -> &T {
        &self.bounds.start
    }

    pub const fn high(&self) -> &T {
        &self.bounds.end
    }

    pub const fn denominator(&self) -> &T {
        &self.denominator
    }
}
