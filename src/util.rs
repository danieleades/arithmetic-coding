pub fn precision(max_denominator: u32) -> u32 {
    let frequency_bits = max_denominator.log2() + 1;
    let minimum_precision = frequency_bits + 2;
    debug_assert!(
        (frequency_bits + minimum_precision) <= u32::BITS,
        "not enough bits to guarantee overflow/underflow avoidance"
    );
    u32::BITS - frequency_bits
}
