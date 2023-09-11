use fenwick_model::simple::FenwickModel;

mod common;

const MAX_DENOMINATOR: u64 = 1 << 20;

#[test]
fn round_trip() {
    let model = FenwickModel::<MAX_DENOMINATOR>::builder(256).build();
    let bytes: &[u8] = &[220, 255, 255];
    let input: Vec<usize> = bytes.iter().copied().map(usize::from).collect();

    common::round_trip(model, &input);
}
