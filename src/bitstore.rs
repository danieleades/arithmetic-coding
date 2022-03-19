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
{
    /// the number of bits needed to represent this type
    const BITS: u32;

    /// the additive identity
    const ZERO: Self;

    /// the multiplicative identity
    const ONE: Self;
}

impl BitStore for u32 {
    const BITS: u32 = u32::BITS;
    const ONE: Self = 1_u32;
    const ZERO: Self = 0_u32;
}
