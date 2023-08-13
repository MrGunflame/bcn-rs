#![no_std]

#[cfg(test)]
extern crate std;

pub mod bc1;

pub type Block8 = [u8; 8];

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
