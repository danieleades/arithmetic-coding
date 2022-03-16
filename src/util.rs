/// Returns the number of bits needed to represent a number
pub const fn bits(integer: u32) -> u32 {
    integer.log2() + 1
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    #[test_case(29 => 5)]
    #[test_case(123 => 7)]
    #[test_case(967 => 10)]
    fn bits(input: u32) -> u32 {
        super::bits(input)
    }
}
