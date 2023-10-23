use bcn::{bc1, Block8};

fn main() {
    if test_bc1(encoded, decoded).is_err() {}
}

fn test_bc1(encoded: &[u8], decoded: &[u8]) -> Result<(), ()> {
    let bc1_block: Block8 = encoded.try_into().unwrap();
    let rgb8_block = decoded;

    let input = bc1::decode(block);
}
