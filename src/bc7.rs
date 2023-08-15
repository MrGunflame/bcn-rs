use crate::bits::BitReader;
use crate::{Block16, Rgba8};

pub fn decode(input: Block16) -> [Rgba8; 16] {
    let mut output = [Rgba8::MIN; 16];

    for x in 0..4 {
        for y in 0..4 {
            output[x as usize * 4 + y as usize] = decode_texel(input, x, y);
        }
    }

    output
}

fn decode_texel(input: Block16, x: u8, y: u8) -> Rgba8 {
    let mut reader = BitReader::new(input);

    let mode = decode_mode(&mut reader);

    // A all-zero bit pattern is invalid. The decoder must
    // return an zeroed block.
    if mode == Mode::Invalid {
        return Rgba8::MIN;
    }

    let mut subset_index = 0;
    let mut num_subsets = 1;
    let partition = 0;

    if mode == Mode::Mode0
        || mode == Mode::Mode1
        || mode == Mode::Mode2
        || mode == Mode::Mode3
        || mode == Mode::Mode7
    {
        num_subsets = get_num_subsets(mode);
        let partition = decode_partition(mode, &mut reader);
        subset_index = get_subset_index(num_subsets, partition, x, y);
    }

    let endpoints = decode_endpoints(mode, &mut reader);

    let endpoint_start = endpoints[2 * subset_index as usize];
    let endpoint_end = endpoints[2 * subset_index as usize];

    let color_index = get_color_index(mode, &mut reader, x, y, num_subsets, partition);
    let color_bitcount = get_color_bitcount(mode);
    // let alpha_index = todo!();
    // let alpha_bitcount = todo!();

    let r = interpolate(
        endpoint_start.r,
        endpoint_end.r,
        color_index,
        color_bitcount,
    );
    let g = interpolate(
        endpoint_start.g,
        endpoint_end.g,
        color_index,
        color_bitcount,
    );
    let b = interpolate(
        endpoint_start.b,
        endpoint_end.b,
        color_index,
        color_bitcount,
    );
    // let a = interpolate(
    //     endpoint_start.a,
    //     endpoint_end.a,
    //     alpha_index,
    //     alpha_bitcount,
    // );
    let a = 255;

    Rgba8 { r, g, b, a }
}

fn decode_mode(reader: &mut BitReader<16>) -> Mode {
    // Eat bits until we find the '1' indicating the mode.

    if reader.read(1) == 1 {
        return Mode::Mode0;
    }

    if reader.read(1) == 1 {
        return Mode::Mode1;
    }

    if reader.read(1) == 1 {
        return Mode::Mode2;
    }

    if reader.read(1) == 1 {
        return Mode::Mode3;
    }

    if reader.read(1) == 1 {
        return Mode::Mode4;
    }

    if reader.read(1) == 1 {
        return Mode::Mode5;
    }

    if reader.read(1) == 1 {
        return Mode::Mode6;
    }

    if reader.read(1) == 1 {
        return Mode::Mode7;
    }

    // The first byte contains an all-zero bit pattern.
    // This not a valid mode.
    Mode::Invalid
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// An invalid all zero mode.
    ///
    /// The decoder must return an all-zero block.
    Invalid,
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Mode6,
    Mode7,
}

fn get_num_subsets(mode: Mode) -> u8 {
    match mode {
        Mode::Mode0 => 3,
        Mode::Mode1 => 2,
        Mode::Mode2 => 3,
        Mode::Mode3 => 2,
        Mode::Mode4 => 1,
        Mode::Mode5 => 1,
        Mode::Mode6 => 1,
        Mode::Mode7 => 2,
        Mode::Invalid => unreachable!(),
    }
}

fn decode_partition(mode: Mode, reader: &mut BitReader<16>) -> u8 {
    match mode {
        Mode::Mode0 => reader.read(4) as u8,
        _ => todo!(),
    }
}

fn get_subset_index(num_subsets: u8, partition_index: u8, x: u8, y: u8) -> u8 {
    match num_subsets {
        // Note that the index is in y-major order.
        2 => PARTITION_SUBSET_2[partition_index as usize][x as usize + y as usize * 4],
        3 => PARTITION_SUBSET_3[partition_index as usize][x as usize + y as usize * 4],
        _ => todo!(),
    }
}

fn decode_endpoints(mode: Mode, reader: &mut BitReader<16>) -> [Rgba8; 6] {
    let mut output = [Rgba8::MIN; 6];

    match mode {
        Mode::Mode0 => {
            for index in 0..6 {
                output[index].r = reader.read(4) as u8;
            }

            for index in 0..6 {
                output[index].g = reader.read(4) as u8;
            }

            for index in 0..6 {
                output[index].b = reader.read(4) as u8;
            }

            // P-bits as LSB
            for color in &mut output {
                let p = (reader.read(1) as u8) << 3;

                // Duplicate the color bits for the lower bits after the P-bit.
                let low_r = color.r >> 1;
                let low_g = color.g >> 1;
                let low_b = color.b >> 1;

                color.r = (color.r << 4) | p | low_r;
                color.g = (color.g << 4) | p | low_g;
                color.b = (color.b << 4) | p | low_b;
            }
        }
        _ => todo!(),
    }

    output
}

fn get_color_index(
    mode: Mode,
    reader: &mut BitReader<16>,
    x: u8,
    y: u8,
    num_subsets: u8,
    partition: u8,
) -> u8 {
    let partition_index = (x as usize + y as usize * 4) as u8;

    let read_len = match num_subsets {
        2 => {
            if partition_index == ANCHOR_INDICES_SUBSET_0[partition as usize] {
                2 - 1
            } else if partition_index == ANCHOR_INDICES_SUBSET_2_2[partition as usize] {
                2 - 1
            } else {
                2
            }
        }
        3 => {
            if partition_index == ANCHOR_INDICES_SUBSET_0[partition as usize] {
                3 - 1
            } else if partition_index == ANCHOR_INDICES_SUBSET_3_2[partition as usize] {
                3 - 1
            } else if partition_index == ANCHOR_INDICES_SUBSET_3_3[partition as usize] {
                3 - 1
            } else {
                3
            }
        }
        _ => todo!(),
    };

    reader.read(read_len) as u8
}

fn get_color_bitcount(mode: Mode) -> u8 {
    match mode {
        Mode::Mode0 => 3,
        _ => todo!(),
    }
}

fn interpolate(start: u8, end: u8, index: u8, index_bitcount: u8) -> u8 {
    match index_bitcount {
        2 => {
            (((64 - WEIGHTS_2[index as usize]) * start as u16
                + WEIGHTS_2[index as usize] * end as u16
                + 32)
                >> 6) as u8
        }
        3 => {
            (((64 - WEIGHTS_3[index as usize]) * start as u16
                + WEIGHTS_3[index as usize] * end as u16)
                >> 6) as u8
        }
        4 => {
            (((64 - WEIGHTS_4[index as usize]) * start as u16
                + WEIGHTS_4[index as usize] * end as u16)
                >> 6) as u8
        }
        _ => unreachable!(),
    }
}

const WEIGHTS_2: [u16; 4] = [0, 21, 43, 64];
const WEIGHTS_3: [u16; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
const WEIGHTS_4: [u16; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];

const PARTITION_SUBSET_2: [[u8; 16]; 64] = [
    [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
    [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
    [0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1],
    [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
    [0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1],
    [0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1],
    [0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
    [0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1],
    [0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0],
    [0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
    [0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1],
    [0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
    [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0],
    [0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0],
    [0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0],
    [0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0],
    [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1],
    [0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0],
    [0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0],
    [0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0],
    [0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1],
    [0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1],
    [0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0],
    [0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0],
    [0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0],
    [0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1],
    [0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1],
    [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0],
    [0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0],
    [0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1],
    [0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1],
    [0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0],
    [0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1],
    [0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1],
    [0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
    [0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0],
    [0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1],
];

const PARTITION_SUBSET_3: [[u8; 16]; 64] = [
    [0, 0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 1, 2, 2, 2, 2],
    [0, 0, 0, 1, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 2, 1],
    [0, 0, 0, 0, 2, 0, 0, 1, 2, 2, 1, 1, 2, 2, 1, 1],
    [0, 2, 2, 2, 0, 0, 2, 2, 0, 0, 1, 1, 0, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2],
    [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 2, 2],
    [0, 0, 2, 2, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2],
    [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
    [0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2],
    [0, 1, 1, 2, 0, 1, 1, 2, 0, 1, 1, 2, 0, 1, 1, 2],
    [0, 1, 2, 2, 0, 1, 2, 2, 0, 1, 2, 2, 0, 1, 2, 2],
    [0, 0, 1, 1, 0, 1, 1, 2, 1, 1, 2, 2, 1, 2, 2, 2],
    [0, 0, 1, 1, 2, 0, 0, 1, 2, 2, 0, 0, 2, 2, 2, 0],
    [0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 2, 1, 1, 2, 2],
    [0, 1, 1, 1, 0, 0, 1, 1, 2, 0, 0, 1, 2, 2, 0, 0],
    [0, 0, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2],
    [0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 1, 1, 1, 1],
    [0, 1, 1, 1, 0, 1, 1, 1, 0, 2, 2, 2, 0, 2, 2, 2],
    [0, 0, 0, 1, 0, 0, 0, 1, 2, 2, 2, 1, 2, 2, 2, 1],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 2, 2, 0, 1, 2, 2],
    [0, 0, 0, 0, 1, 1, 0, 0, 2, 2, 1, 0, 2, 2, 1, 0],
    [0, 1, 2, 2, 0, 1, 2, 2, 0, 0, 1, 1, 0, 0, 0, 0],
    [0, 0, 1, 2, 0, 0, 1, 2, 1, 1, 2, 2, 2, 2, 2, 2],
    [0, 1, 1, 0, 1, 2, 2, 1, 1, 2, 2, 1, 0, 1, 1, 0],
    [0, 0, 0, 0, 0, 1, 1, 0, 1, 2, 2, 1, 1, 2, 2, 1],
    [0, 0, 2, 2, 1, 1, 0, 2, 1, 1, 0, 2, 0, 0, 2, 2],
    [0, 1, 1, 0, 0, 1, 1, 0, 2, 0, 0, 2, 2, 2, 2, 2],
    [0, 0, 1, 1, 0, 1, 2, 2, 0, 1, 2, 2, 0, 0, 1, 1],
    [0, 0, 0, 0, 2, 0, 0, 0, 2, 2, 1, 1, 2, 2, 2, 1],
    [0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 2, 2, 2],
    [0, 2, 2, 2, 0, 0, 2, 2, 0, 0, 1, 2, 0, 0, 1, 1],
    [0, 0, 1, 1, 0, 0, 1, 2, 0, 0, 2, 2, 0, 2, 2, 2],
    [0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0],
    [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0],
    [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0],
    [0, 1, 2, 0, 2, 0, 1, 2, 1, 2, 0, 1, 0, 1, 2, 0],
    [0, 0, 1, 1, 2, 2, 0, 0, 1, 1, 2, 2, 0, 0, 1, 1],
    [0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0, 1, 1],
    [0, 1, 0, 1, 0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2],
    [0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 1, 2, 1, 2, 1],
    [0, 0, 2, 2, 1, 1, 2, 2, 0, 0, 2, 2, 1, 1, 2, 2],
    [0, 0, 2, 2, 0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 1, 1],
    [0, 2, 2, 0, 1, 2, 2, 1, 0, 2, 2, 0, 1, 2, 2, 1],
    [0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 1, 0, 1],
    [0, 0, 0, 0, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1],
    [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 2, 2, 2, 2],
    [0, 2, 2, 2, 0, 1, 1, 1, 0, 2, 2, 2, 0, 1, 1, 1],
    [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2],
    [0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2],
    [0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 2, 2, 2],
    [0, 0, 0, 2, 1, 1, 1, 2, 1, 1, 1, 2, 0, 0, 0, 2],
    [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 2, 2],
    [0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 1, 2],
    [0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 2, 2, 2, 2, 2, 2],
    [0, 0, 2, 2, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 2, 2],
    [0, 0, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2, 0, 0, 2, 2],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2],
    [0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1],
    [0, 2, 2, 2, 1, 2, 2, 2, 0, 2, 2, 2, 1, 2, 2, 2],
    [0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
    [0, 1, 1, 1, 2, 0, 1, 1, 2, 2, 0, 1, 2, 2, 2, 0],
];

const ANCHOR_INDICES_SUBSET_0: [u8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, //
];

#[rustfmt::skip]
const ANCHOR_INDICES_SUBSET_2_2: [u8; 64] = [
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 02, 08, 02, 02, 08, 08, 15,
    02, 08, 02, 02, 08, 08, 02, 02,
    15, 15, 06, 08, 02, 08, 15, 15,
    02, 08, 02, 02, 02, 15, 15, 06,
    06, 02, 06, 08, 15, 15, 02, 02,
    15, 15, 15, 15, 15, 02, 02, 15,
];

const ANCHOR_INDICES_SUBSET_3_2: [u8; 64] = [
    3, 3, 15, 15, 8, 3, 15, 15, //
    8, 8, 6, 6, 6, 5, 3, 3, //
    3, 3, 8, 15, 3, 3, 6, 10, //
    5, 8, 8, 6, 8, 5, 15, 15, //
    8, 15, 3, 5, 6, 10, 8, 15, //
    15, 3, 15, 5, 15, 15, 15, 15, //
    3, 15, 5, 5, 5, 8, 5, 10, //
    5, 10, 8, 13, 15, 12, 3, 3, //
];

const ANCHOR_INDICES_SUBSET_3_3: [u8; 64] = [
    15, 8, 8, 3, 15, 15, 3, 8, //
    15, 15, 15, 15, 15, 15, 15, 8, //
    15, 8, 15, 3, 15, 8, 15, 8, //
    3, 15, 6, 10, 15, 15, 10, 8, //
    15, 3, 15, 10, 10, 8, 9, 10, //
    6, 15, 8, 15, 3, 6, 6, 8, //
    15, 3, 15, 15, 15, 15, 15, 15, //
    15, 15, 15, 15, 3, 15, 15, 8, //
];
