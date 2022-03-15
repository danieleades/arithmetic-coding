use std::ops::Range;

pub struct Probability<T> {
    bounds: Range<T>,
    denominator: T,
}

impl<T> Probability<T>
where
    T: PartialOrd,
{
    pub fn new(bounds: Range<T>, denominator: T) -> Self {
        Self {
            bounds,
            denominator,
        }
    }

    pub fn low(&self) -> &T {
        &self.bounds.start
    }

    pub fn high(&self) -> &T {
        &self.bounds.end
    }

    pub fn denominator(&self) -> &T {
        &self.denominator
    }
}
