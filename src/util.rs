use crate::BitStore;

pub fn precision<B: BitStore>(max_denominator: B) -> u32 {
    let frequency_bits = max_denominator.log2() + 1;
    let minimum_precision = frequency_bits + 2;
    debug_assert!(
        (frequency_bits + minimum_precision) <= B::BITS,
        "not enough bits to guarantee overflow/underflow avoidance"
    );
    B::BITS - frequency_bits
}
