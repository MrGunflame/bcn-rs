use crate::{read_u16_le, Block8, Rgb8};

#[derive(Debug)]
struct Table {
    colors: [Rgb8; 4],
}

impl Table {
    fn new(colors: [Rgb8; 2]) -> Self {
        let r0 = colors[0].r as f32;
        let g0 = colors[0].g as f32;
        let b0 = colors[0].b as f32;
        let r1 = colors[1].r as f32;
        let g1 = colors[1].g as f32;
        let b1 = colors[1].b as f32;

        let c2 = Rgb8 {
            r: (r0 * 2.0 / 3.0 + r1 * 1.0 / 3.0) as u8,
            g: (g0 * 2.0 / 3.0 + g1 * 1.0 / 3.0) as u8,
            b: (b0 * 2.0 / 3.0 + b1 * 1.0 / 3.0) as u8,
        };
        let c3 = Rgb8 {
            r: (r0 * 1.0 / 3.0 + r1 * 2.0 / 3.0) as u8,
            g: (g0 * 1.0 / 3.0 + g1 * 2.0 / 3.0) as u8,
            b: (b0 * 1.0 / 3.0 + b1 * 2.0 / 3.0) as u8,
        };

        Self {
            colors: [colors[0], colors[1], c2, c3],
        }
    }

    fn get(&self, index: u8) -> Rgb8 {
        debug_assert!(index <= 0b11);
        self.colors[index as usize]
    }

    fn closest(&self, color: Rgb8) -> u8 {
        let mut index = 0;
        let mut distance = i32::MAX;

        for (i, c) in self.colors.iter().enumerate() {
            let delta = color.distance(*c).abs();
            if delta < distance {
                index = i;
                distance = delta;
            }
        }

        debug_assert!(index <= 0b11);
        index as u8
    }
}

pub fn encode(input: [Rgb8; 16]) -> Block8 {
    let (min, max) = find_min_max(input);

    let c0 = encode_565_rgb(min.r, min.g, min.b);
    let c1 = encode_565_rgb(max.r, max.g, max.b);

    let table = Table::new([min, max]);

    let mut output = [0; 8];
    output[0] = c0.to_le_bytes()[0];
    output[1] = c0.to_le_bytes()[1];
    output[2] = c1.to_le_bytes()[0];
    output[3] = c1.to_le_bytes()[1];

    for (row, chunk) in input.chunks(4).enumerate() {
        let f0 = table.closest(chunk[0]);
        let f1 = table.closest(chunk[1]);
        let f2 = table.closest(chunk[2]);
        let f3 = table.closest(chunk[3]);

        let byte = (f0 << 6) | (f1 << 4) | (f2 << 2) | f3;
        output[row + 4] = byte;
    }

    output
}

/// Decode a single BC1 block.
pub fn decode(input: Block8) -> [Rgb8; 16] {
    let c0 = decode_565_rgb(read_u16_le(input[0], input[1]));
    let c1 = decode_565_rgb(read_u16_le(input[2], input[3]));

    let table = Table::new([c0, c1]);

    let mut output = [Rgb8::from_arry([0; 3]); 16];
    for row in 0..4 {
        let byte = input[row + 4];

        let f0 = (byte & 0b1100_0000) >> 6;
        let f1 = (byte & 0b0011_0000) >> 4;
        let f2 = (byte & 0b0000_1100) >> 2;
        let f3 = byte & 0b0000_0011;

        output[row * 4] = table.get(f0);
        output[row * 4 + 1] = table.get(f1);
        output[row * 4 + 2] = table.get(f2);
        output[row * 4 + 3] = table.get(f3);
    }

    output
}

fn encode_565_rgb(r: u8, g: u8, b: u8) -> u16 {
    let r = ((r / 4) as u16) << (5 + 6);
    let g = ((g / 2) as u16) << 5;
    let b = (b / 4) as u16;

    r | g | b
}

fn decode_565_rgb(rgb: u16) -> Rgb8 {
    let r = rgb >> (5 + 6);
    let g = (rgb >> 5) & 0b0011_1111;
    let b = rgb & 0b0001_1111;

    Rgb8 {
        r: r as u8 * 4,
        g: g as u8 * 2,
        b: b as u8 * 4,
    }
}

fn find_min_max(input: [Rgb8; 16]) -> (Rgb8, Rgb8) {
    let mut min = u16::MAX;
    let mut max = 0;

    let mut min_color = Rgb8::MAX;
    let mut max_color = Rgb8::MIN;

    for color in input {
        let val = color.r as u16 + color.g as u16 * 2 + color.b as u16;

        if val < min {
            min = val;
            min_color = color;
        }

        if val > max {
            max = val;
            max_color = color;
        }
    }

    (min_color, max_color)
}

#[cfg(test)]
mod tests {
    use crate::Rgb8;

    use super::{decode, encode};

    #[test]
    fn bc1_encode() {
        let input = [
            Rgb8 { r: 0, g: 1, b: 2 },
            Rgb8 { r: 3, g: 4, b: 5 },
            Rgb8 { r: 6, g: 7, b: 8 },
            Rgb8 { r: 9, g: 11, b: 12 },
            Rgb8 { r: 0, g: 1, b: 2 },
            Rgb8 { r: 0, g: 1, b: 2 },
            Rgb8 { r: 3, g: 4, b: 5 },
            Rgb8 { r: 6, g: 7, b: 8 },
            Rgb8 { r: 9, g: 11, b: 12 },
            Rgb8 { r: 3, g: 4, b: 5 },
            Rgb8 { r: 6, g: 7, b: 8 },
            Rgb8 { r: 9, g: 11, b: 12 },
            Rgb8 { r: 0, g: 1, b: 2 },
            Rgb8 { r: 3, g: 4, b: 5 },
            Rgb8 { r: 6, g: 7, b: 8 },
            Rgb8 { r: 9, g: 11, b: 12 },
        ];

        let block = encode(input);
        assert_eq!(
            block,
            [
                0b0000_0000,
                0b0000_0000,
                0b1010_0011,
                0b0001_0000,
                0b0010_1101,
                0b0000_1011,
                0b0110_1101,
                0b0010_1101,
            ]
        );
    }

    #[test]
    fn bc1_decode() {
        let input = [
            0b0000_0000,
            0b0000_0000,
            0b1010_0011,
            0b0001_0000,
            0b0010_1101,
            0b0000_1011,
            0b0110_1101,
            0b0010_1101,
        ];

        let output = decode(input);
        assert_eq!(
            output,
            [
                Rgb8 { r: 0, g: 0, b: 0 },
                Rgb8 { r: 2, g: 3, b: 4 },
                Rgb8 { r: 5, g: 6, b: 8 },
                Rgb8 { r: 8, g: 10, b: 12 },
                Rgb8 { r: 0, g: 0, b: 0 },
                Rgb8 { r: 0, g: 0, b: 0 },
                Rgb8 { r: 2, g: 3, b: 4 },
                Rgb8 { r: 5, g: 6, b: 8 },
                Rgb8 { r: 8, g: 10, b: 12 },
                Rgb8 { r: 2, g: 3, b: 4 },
                Rgb8 { r: 5, g: 6, b: 8 },
                Rgb8 { r: 8, g: 10, b: 12 },
                Rgb8 { r: 0, g: 0, b: 0 },
                Rgb8 { r: 2, g: 3, b: 4 },
                Rgb8 { r: 5, g: 6, b: 8 },
                Rgb8 { r: 8, g: 10, b: 12 },
            ]
        );
    }
}
