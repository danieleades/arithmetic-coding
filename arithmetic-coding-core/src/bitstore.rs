use std::ops::{Add, AddAssign, Div, Mul, Shl, ShlAssign, Sub};

/// A trait for a type that can be used for the internal integer representation
/// of an encoder or decoder
pub trait BitStore:
    Shl<u32, Output = Self>
    + ShlAssign<u32>
    + Sized
    + From<u32>
    + Sub<Output = Self>
    + Add<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + PartialOrd
    + Copy
    + std::fmt::Debug
{
    /// the number of bits needed to represent this type
    const BITS: u32;

    /// the additive identity
    const ZERO: Self;

    /// the multiplicative identity
    const ONE: Self;

    /// integer natural logarithm, rounded down
    fn log2(self) -> u32;
}

impl BitStore for u32 {
    const BITS: u32 = u32::BITS;
    const ONE: Self = 1;
    const ZERO: Self = 0;

    fn log2(self) -> u32 {
        u32::ilog2(self)
    }
}

impl BitStore for u64 {
    const BITS: u32 = u64::BITS as u32;
    const ONE: Self = 1;
    const ZERO: Self = 0;

    fn log2(self) -> u32 {
        u64::ilog2(self)
    }
}

impl BitStore for u128 {
    const BITS: u32 = u128::BITS as u32;
    const ONE: Self = 1;
    const ZERO: Self = 0;

    fn log2(self) -> u32 {
        u128::ilog2(self)
    }
}
