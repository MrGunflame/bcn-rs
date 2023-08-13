use crate::{bc1, Block16, Rgba8};

/// Encode 16 texels into a single BC2 block.
pub fn encode(input: [Rgba8; 16]) -> Block16 {
    let mut output = [0; 16];

    for row in 0..4 {
        let a0 = input[row * 4].a / 16;
        let a1 = input[row * 4 + 1].a / 16;
        let a2 = input[row * 4 + 2].a / 16;
        let a3 = input[row * 4 + 3].a / 16;

        debug_assert!(a0 <= 0b1111);
        debug_assert!(a1 <= 0b1111);
        debug_assert!(a2 <= 0b1111);
        debug_assert!(a3 <= 0b1111);

        output[row * 2] = a0 | a1;
        output[row * 2 + 1] = a2 | a3;
    }

    // Color section has the same format as BC1.
    let rgb = input.map(|c| c.to_rgb8());
    let color_section = bc1::encode(rgb);
    output[8..].copy_from_slice(&color_section);

    output
}

/// Decode a single BC2 block.
pub fn decode(input: Block16) -> [Rgba8; 16] {
    let mut output = [Rgba8::from_array([0; 4]); 16];

    for row in 0..4 {
        let a0 = (input[row * 2] >> 4) * 16;
        let a1 = (input[row * 2] & 0b1111) * 16;
        let a2 = (input[row * 2 + 1] >> 4) * 16;
        let a3 = (input[row * 2 + 1] & 0b1111) * 16;

        output[row * 4].a = a0;
        output[row * 4 + 1].a = a1;
        output[row * 4 + 2].a = a2;
        output[row * 4 + 3].a = a3;
    }

    let colors = bc1::decode(input[8..].try_into().unwrap());
    for (i, c) in colors.into_iter().enumerate() {
        output[i].r = c.r;
        output[i].g = c.g;
        output[i].b = c.b;
    }

    output
}
