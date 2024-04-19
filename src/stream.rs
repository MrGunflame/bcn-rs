use alloc::vec::Vec;
use image::{ImageBuffer, Rgba};

use crate::bc1::Bc1;
use crate::{Decoder, Rgba8};

#[derive(Debug)]
pub struct StreamDecoder<D> {
    _decoder: D,
}

impl<D> StreamDecoder<D> {
    pub fn new(decoder: D) -> Self {
        Self { _decoder: decoder }
    }

    pub fn decode(&self, mut buf: &[u8], width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>>
    where
        D: Decoder,
    {
        assert!(width % D::BLOCK_SIZE as u32 == 0);
        assert!(height % D::BLOCK_SIZE as u32 == 0);

        assert_eq!(
            buf.len(),
            width as usize * height as usize / (D::NUM_PIXELS * D::NUM_PIXELS / D::BLOCK_SIZE)
        );

        let mut current_width = 0;
        let mut current_height = 0;

        let mut img = ImageBuffer::new(width, height);

        while current_width < width && current_height < height {
            let block = &buf[0..D::BLOCK_SIZE];
            buf = &buf[D::BLOCK_SIZE..];

            let mut output = [Rgba8::MIN; 32];
            D::decode(block, &mut output);

            for (index, pixel) in output.into_iter().enumerate() {
                if index >= D::NUM_PIXELS * D::NUM_PIXELS {
                    break;
                }

                let offset_w = (index % 4) as u32;
                let offset_h = (index / 4) as u32;

                img.put_pixel(
                    current_width + offset_w,
                    current_height + offset_h,
                    Rgba([pixel.r, pixel.g, pixel.b, pixel.a]),
                );
            }

            current_width += 4;
            if current_width == width {
                current_width = 0;
                current_height += 4;
            }
        }

        img
    }
}

pub fn decode_bc1_stream(
    mut buf: &[u8],
    width: u32,
    height: u32,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    // assert!(width % 4 == 0);
    // assert!(height % 4 == 0);

    // // BC1 converts 8 byte blocks into 16 pixels each.
    // assert_eq!(buf.len(), width as usize * height as usize / 2);

    // let mut current_width = 0;
    // let mut current_height = 0;

    // let mut img = ImageBuffer::new(width, height);

    // while current_width < width && current_height < height {
    //     let block = buf[0..8].try_into().unwrap();
    //     buf = &buf[8..];

    //     let output = crate::bc1::decode(block);
    //     for (index, pixel) in output.into_iter().enumerate() {
    //         let offset_w = 3 - (index % 4) as u32;
    //         let offset_h = (index / 4) as u32;

    //         img.put_pixel(
    //             current_width + offset_w,
    //             current_height + offset_h,
    //             Rgb([pixel.r, pixel.g, pixel.b]),
    //         );
    //     }

    //     current_width += 4;
    //     if current_width == width {
    //         current_width = 0;
    //         current_height += 4;
    //     }
    // }

    // img
    StreamDecoder::new(Bc1).decode(buf, width, height)
}
