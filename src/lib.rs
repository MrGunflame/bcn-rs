#![no_std]

extern crate alloc;

//#[cfg(test)]
//extern crate std;
extern crate std;

pub mod bc1;
pub mod bc2;
pub mod bc7;
pub mod stream;

mod bits;

pub type Block8 = [u8; 8];
pub type Block16 = [u8; 16];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rgba8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba8 {
    const MIN: Self = Self {
        r: u8::MIN,
        g: u8::MIN,
        b: u8::MIN,
        a: u8::MIN,
    };

    #[inline]
    pub const fn from_array(v: [u8; 4]) -> Self {
        Self {
            r: v[0],
            g: v[1],
            b: v[2],
            a: v[3],
        }
    }

    #[inline]
    fn to_rgb8(self) -> Rgb8 {
        Rgb8 {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb8 {
    const MIN: Self = Self {
        r: u8::MIN,
        g: u8::MIN,
        b: u8::MIN,
    };

    const MAX: Self = Self {
        r: u8::MAX,
        g: u8::MAX,
        b: u8::MAX,
    };

    #[inline]
    pub const fn from_arry(v: [u8; 3]) -> Self {
        Self {
            r: v[0],
            g: v[1],
            b: v[2],
        }
    }

    fn distance(self, other: Self) -> i32 {
        let r = self.r as i32 - other.r as i32;
        let g = self.g as i32 - other.g as i32;
        let b = self.b as i32 - other.b as i32;

        r * r + g * g + b * b
    }
}

fn read_u16_le(a: u8, b: u8) -> u16 {
    u16::from_le_bytes([a, b])
}

pub trait Encoder: private::Sealed {}

pub trait Decoder: private::Sealed {}

mod private {
    use crate::Rgba8;

    pub trait Sealed {
        /// Input block size.
        const BLOCK_SIZE: usize;

        /// Number of pixels in both direction for each block.
        const NUM_PIXELS: usize;

        /// decode(&[u8; Self::BLOCK_SIZE], out: &mut [Rgba8; Self::NUM_PIXELS * Self::NUM_PIXELS]);
        fn decode(block: &[u8], out: &mut [Rgba8]);
    }
}

#[cfg(test)]
mod tests {
    use crate::Rgb8;

    #[test]
    fn distance_zero() {
        let lhs = Rgb8::MIN;
        let rhs = Rgb8::MIN;

        assert_eq!(lhs.distance(rhs), 0);
    }

    #[test]
    fn distance_min_max() {
        let lhs = Rgb8::MIN;
        let rhs = Rgb8::MAX;

        assert_eq!(lhs.distance(rhs), (255 * 255) * 3);
    }
}
